use serde::Deserialize;
use std::{path::PathBuf, str::FromStr};

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "project_root")]
    pub project_root: PathBuf,

    #[serde(default = "git_dir")]
    pub git_dir: PathBuf,

    #[serde(default)]
    pub jj_dirs: Vec<PathBuf>,

    pub gerrit_server: String,

    pub repo: String,

    #[serde(default = "default_events")]
    pub events: Vec<String>,
}

fn project_root() -> PathBuf {
    std::env::current_dir().expect("failed to get current working directory")
}

fn git_dir() -> PathBuf {
    PathBuf::from_str(".git").unwrap()
}

fn default_events() -> Vec<String> {
    vec![
        "change-abandoned".to_string(),
        "change-deleted".to_string(),
        "change-merged".to_string(),
        "change-restored".to_string(),
        "dropped-output".to_string(),
        "comment-added".to_string(),
        "patchset-created".to_string(),
        "ref-updated".to_string(),
        "batch-ref-updated".to_string(),
        "reviewer-added".to_string(),
        "reviewer-deleted".to_string(),
        "topic-changed".to_string(),
        "wip-state-changed".to_string(),
        "private-state-changed".to_string(),
        "vote-deleted".to_string(),
    ]
}

impl Config {
    pub fn load(path: &std::path::Path) -> &'static Self {
        let text = std::fs::read_to_string(path).unwrap();
        let mut cfg: Config = toml::from_str(&text).unwrap();
        cfg.git_dir = cfg.project_root.join(cfg.git_dir);

        cfg.jj_dirs = match cfg.jj_dirs.is_empty() {
            true => vec![cfg.project_root.clone()],
            false => cfg
                .jj_dirs
                .into_iter()
                .map(|path| cfg.project_root.join(path))
                .collect(),
        };
        println!("{:?}", cfg);
        Box::leak(Box::new(cfg))
    }
}
