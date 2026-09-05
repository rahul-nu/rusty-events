use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub gerrit_server: String,
    pub project_root: PathBuf,
    pub repo: String,
    #[serde(default = "origin")]
    pub remote_name: String,
    #[serde(default = "default_events")]
    pub events: Vec<String>,
}
fn origin() -> String {
    "origin".to_owned()
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
    pub fn load(path: &std::path::Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading config file at {}", path.display()))?;
        let cfg: Config = toml::from_str(&text)
            .with_context(|| format!("parsing config file at {}", path.display()))?;
        Ok(cfg)
    }
}
