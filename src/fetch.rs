use anyhow::{Context, Ok, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{self, Sender};
use std::thread::JoinHandle;

use crate::events::GerritEventType;

pub struct FetchRequest {
    pub repo_dir: PathBuf,
    pub remote: String,
    pub event: GerritEventType,
}

pub struct FetchWorker {
    tx: Sender<FetchRequest>,
    handle: Option<JoinHandle<()>>,
}

impl FetchWorker {
    /// Spawns a single background thread that processes fetch requests
    /// strictly one at a time, in the order they were enqueued.
    pub fn spawn() -> Self {
        let (tx, rx) = mpsc::channel::<FetchRequest>();

        let handle = std::thread::spawn(move || {
            for req in rx {
                if let Err(e) = fetch_ref(&req.repo_dir, &req.remote, req.event) {
                    eprintln!("warning: {e:#}");
                }
            }
            // Channel closed (all senders dropped) -> worker exits.
        });

        FetchWorker {
            tx,
            handle: Some(handle),
        }
    }

    /// Enqueues a fetch. Never blocks the caller on git itself — only on
    /// pushing to the channel, which is effectively instant.
    pub fn enqueue(&self, req: FetchRequest) {
        // An error here only happens if the worker thread has died, e.g.
        // it panicked. We don't want to crash the event loop over that;
        // just log it.
        if self.tx.send(req).is_err() {
            eprintln!("warning: fetch worker is no longer running, dropped a fetch request");
        }
    }

    /// Waits for all currently-queued fetches to finish, then joins the
    /// worker thread. Call this on clean shutdown if you want to drain
    /// the queue rather than abandon it.
    pub fn shutdown(mut self) {
        drop(self.tx); // closes the channel, worker's `for req in rx` loop ends
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

fn fetch_ref(repo_dir: &Path, remote: &str, event: GerritEventType) -> Result<()> {
    match event {
        GerritEventType::PatchsetCreated { patch_set, .. } => {
            let ref_name = patch_set.ref_name;
            let (_, name) = ref_name.split_at_checked(5).unwrap();
            let output = Command::new("git")
                .arg("fetch")
                .arg(remote)
                .arg(format!("{ref_name}:{name}"))
                .current_dir(repo_dir)
                .output()
                .with_context(|| {
                    format!(
                        "spawning `git fetch {remote} {ref_name}` in {}",
                        repo_dir.display()
                    )
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("git fetch {ref_name} failed: {stderr}");
            }
            let output = Command::new("jj")
                .arg("tag")
                .arg("set")
                .arg("-r")
                .arg(name)
                .arg(name)
                .current_dir(repo_dir)
                .output()
                .with_context(|| {
                    format!(
                        "spawning `git fetch {remote} {ref_name}` in {}",
                        repo_dir.display()
                    )
                })?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("jj tag set -r {ref_name} {ref_name}: {stderr}");
            }
            let output = Command::new("jj")
                .arg("bookmark")
                .arg("forget")
                .arg(name)
                .current_dir(repo_dir)
                .output()
                .with_context(|| {
                    format!(
                        "spawning `git fetch {remote} {ref_name}` in {}",
                        repo_dir.display()
                    )
                })?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("jj bookmark forget {ref_name} failed: {stderr}");
            }
            let output = Command::new("jj")
                .arg("git")
                .arg("fetch")
                .current_dir(repo_dir)
                .output()
                .with_context(|| "spawning `jj git fetch to clean out")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("jj bookmark forget {ref_name} failed: {stderr}");
            }

            if output.status.success() {
                eprintln!("fetched {ref_name}");
            }
            Ok(())
        }
        GerritEventType::ChangeMerged { .. } => {
            let output = Command::new("jj")
                .arg("git")
                .arg("fetch")
                .current_dir(repo_dir)
                .output()
                .with_context(|| "spawning `jj git fetch to clean out")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("jj git fetch failed: {stderr}");
            };
            Ok(())
        }
        _ => Ok(()),
    }
}
