mod config;
mod events;

use config::Config;
use events::GerritEvent;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use threadpool::ThreadPool;

static DELETE_LIST_LOCK: Mutex<()> = Mutex::new(());

fn main() {
    let config_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("config.toml"));

    let config: &'static Config = Box::leak(Box::new(Config::load(&config_path).unwrap()));
    let pool = ThreadPool::new(4);
    let mut cmd = Command::new("ssh");
    cmd.arg(&config.gerrit_server)
        .arg("gerrit")
        .arg("stream-events");

    for event in &config.events {
        cmd.arg("-s").arg(event);
    }

    let mut child = cmd.stdout(Stdio::piped()).spawn().unwrap();
    pool.execute(|| full_sync(config));

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        let line = line.unwrap();
        println!("{line}");
        match serde_json::from_str::<GerritEvent>(&line) {
            Ok(event) => {
                pool.execute(move || {
                    handle_event(event, config);
                });
            }
            Err(_) => {
                eprintln!("warning: failed to deserialize event");
                println!("{line}");
            }
        };
    }
    child.wait().unwrap();

    eprintln!("gerrit stream-events channel closed.");
}

fn handle_event(event: GerritEvent, cfg: &Config) {
    let project = match event.project() {
        None => return,
        Some(project) => project,
    };
    if project != cfg.repo.as_str() {
        return;
    }
    let bare_git = cfg.project_root.join("bare.git");
    println!("{:?}", event.project());
    match event {
        GerritEvent::ChangeAbandoned { patch_set, .. } => append_to_file(
            &(cfg.project_root.join("delete_list.txt")),
            &patch_set.ref_string,
        ),
        GerritEvent::ChangeMerged { patch_set, .. } => {
            fetch(&"main".to_string(), &bare_git);
            append_to_file(
                &(cfg.project_root.join("delete_list.txt")),
                &patch_set.ref_string,
            )
        }
        GerritEvent::PatchsetCreated { patch_set, .. }
        | GerritEvent::ChangeRestored { patch_set, .. } => {
            let (change_num, _) = patch_set.split().unwrap();

            let bare_git = cfg.project_root.join("bare.git");
            setup_workspace(cfg, change_num, &bare_git);
            fetch(&patch_set.ref_string, &bare_git);
            let workspace = cfg.project_root.join(format!("change_{}", change_num));
            sync(&workspace)
        }
        GerritEvent::DroppedOutput
        | GerritEvent::RefUpdated { .. }
        | GerritEvent::BatchRefUpdated { .. } => {
            full_sync(cfg);
        }
        _ => (),
    }
}

#[allow(unused)]
fn full_sync(cfg: &Config) {
    let bare_git = cfg.project_root.join("bare.git");
    assert!(
        Command::new("git")
            .arg(format!("--git-dir={}", bare_git.display()))
            .arg("fetch")
            .arg("origin")
            .status()
            .unwrap()
            .success()
    );
}

fn append_to_file(file: &Path, data: &String) {
    let _guard = DELETE_LIST_LOCK.lock().unwrap();
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(file)
        .unwrap();

    writeln!(f, "{}", data).unwrap();
}

fn setup_workspace(cfg: &Config, change_num: u32, bare_git: &Path) {
    let workspace = &cfg.project_root.join(format!("change_{}", change_num));
    if workspace.exists() {
        return;
    }
    println!("Creating jj repo: {}", workspace.display());
    assert!(
        Command::new("jj")
            .arg("git")
            .arg("init")
            .arg(workspace)
            .arg(format!("--git-repo={}", bare_git.display()))
            .status()
            .unwrap()
            .success()
    );
    let remote_name = &cfg.remote_name;
    let two_dig = change_num % 100;
    let revset = format!(
        r#"fork_point(main@{remote_name} | mutable() | tags("changes/{two_dig}/{change_num}/*")) | main@{remote_name} | mutable() | tags("changes/{two_dig}/{change_num}/*")"#
    );
    let toml_value = format!("'{}'", revset);

    assert!(
        Command::new("jj")
            .current_dir(workspace)
            .args(["config", "set", "--repo", "revsets.log", &toml_value])
            .status()
            .unwrap()
            .success()
    );
}

fn fetch(ref_to_fetch: &String, bare_git: &Path) {
    assert!(
        Command::new("git")
            .arg(format!("--git-dir={}", bare_git.display()))
            .arg("fetch")
            .arg("origin")
            .arg(ref_to_fetch)
            .status()
            .unwrap()
            .success()
    );
}

fn sync(workspace_dir: &Path) {
    assert!(
        Command::new("jj")
            .arg("-R")
            .arg(workspace_dir)
            .args(["git", "import"])
            .status()
            .unwrap()
            .success()
    );

    let home = std::env::var("HOME").unwrap();
    let layout = Path::new(&home).join(".config/zellij/layouts/repo.kdl");

    let parent_name = workspace_dir
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("");

    let ws_name = workspace_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    let session_name = format!("{parent_name}::{ws_name}");

    assert!(
        Command::new("zellij")
            .current_dir(workspace_dir)
            .arg("attach")
            .arg("--create-background")
            .arg(&session_name)
            .arg("options")
            .arg("--default-layout")
            .arg(&layout)
            .status()
            .unwrap()
            .success()
    );
}
