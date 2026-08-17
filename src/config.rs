use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub username: String,
    pub private_key: PathBuf,
    pub public_key: Option<PathBuf>,
    pub passphrase: Option<String>,
    /// Event types to subscribe to, e.g. ["patchset-created", "comment-added"].
    /// Empty or omitted means subscribe to all events.
    #[serde(default)]
    pub subscribe: Vec<String>,
    pub project: String,
    /// Local path to a git repo/bare mirror where refs should be fetched into.
    #[serde(default = "default_remote")]
    pub git_remote: String,
    pub git_repo: PathBuf,
}

fn default_port() -> u16 {
    29418
}

fn default_remote() -> String {
    "origin".to_string()
}

impl Config {
    pub fn load(path: &std::path::Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading config file at {}", path.display()))?;
        let cfg: Config = toml::from_str(&text)
            .with_context(|| format!("parsing config file at {}", path.display()))?;
        cfg.validate()?;
        Ok(cfg)
    }

    fn validate(&self) -> Result<()> {
        for event in &self.subscribe {
            if event.is_empty() || !event.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                anyhow::bail!(
                    "invalid entry in `subscribe`: {event:?} (expected an event type like \"patchset-created\")"
                );
            }
        }
        Ok(())
    }

    /// Builds the `gerrit stream-events [-s ...]` command to exec over SSH.
    pub fn stream_events_command(&self) -> String {
        let mut cmd = String::from("gerrit stream-events");
        for event in &self.subscribe {
            cmd.push_str(" -s ");
            cmd.push_str(event);
        }
        cmd
    }
}
