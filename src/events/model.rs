use super::approval::{Account, Approval};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeKey {
    pub id: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Change {
    pub project: String,
    pub branch: String,
    pub topic: Option<String>,
    pub id: String,
    pub subject: String,
    pub owner: Account,
    pub url: String,
    pub commit_message: String,
    pub hashtags: Option<Vec<String>>,
    pub created_on: i64,
    pub last_updated: Option<i64>,
    pub status: Status,
    pub private: Option<bool>,
    pub wip: Option<bool>,
    // pub patch_sets: Vec<PatchSet>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Status {
    New,
    Merged,
    Abandoned,
    NoChange,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchSet {
    pub number: u16,
    pub revision: String,
    #[serde(default)]
    pub parents: Vec<String>,
    #[serde(rename = "ref")]
    pub ref_string: String,
    pub uploader: Account,
    pub author: Option<Account>,
    pub created_on: i64,
    pub kind: Option<PatchSetKind>,
    #[serde(default)]
    pub approvals: Vec<Approval>,
    // pub size_insertions: Option<i64>,
    // pub size_deletions: Option<i64>,
}
impl PatchSet {
    pub fn split(&self) -> Result<(u32, u16)> {
        let mut parts = self.ref_string.split('/').skip(3);

        let change_num: u32 = parts
            .next()
            .context("missing second segment")?
            .parse()
            .context("failed to parse second segment as u32")?;

        let patch_num: u16 = parts
            .next()
            .context("missing third segment")?
            .parse()
            .context("failed to parse third segment as u16")?;

        Ok((change_num, patch_num))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PatchSetKind {
    Rework,
    TrivialRebase,
    TrivialRebaseWithMessageUpdate,
    MergeFirstParentUpdate,
    NoCodeChange,
    NoChange,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefUpdate {
    pub old_rev: String,
    /// All-zero SHA indicates the ref was deleted.
    pub new_rev: String,
    pub ref_name: String,
    pub project: String,
}
