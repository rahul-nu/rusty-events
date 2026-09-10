use super::approval::{Account, Approval};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeKey<'a> {
    #[serde(borrow)]
    pub id: &'a str,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Change<'a> {
    #[serde(borrow)]
    pub project: &'a str,
    #[serde(borrow)]
    pub branch: &'a str,
    #[serde(borrow)]
    pub topic: Option<&'a str>,
    #[serde(borrow)]
    pub id: &'a str,
    #[serde(borrow)]
    pub subject: &'a str,
    pub number: u32,
    #[serde(borrow)]
    pub owner: Account<'a>,
    #[serde(borrow)]
    pub url: &'a str,
    #[serde(borrow)]
    pub commit_message: &'a str,
    #[serde(borrow)]
    pub hashtags: Option<Vec<&'a str>>,
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
pub struct PatchSet<'a> {
    pub number: u16,
    #[serde(borrow)]
    pub revision: &'a str,
    #[serde(borrow)]
    #[serde(default)]
    pub parents: Vec<&'a str>,
    #[serde(rename = "ref")]
    #[serde(borrow)]
    pub ref_string: &'a str,
    #[serde(borrow)]
    pub uploader: Account<'a>,
    #[serde(borrow)]
    pub author: Option<Account<'a>>,
    pub created_on: i64,
    pub kind: Option<PatchSetKind>,
    #[serde(default)]
    #[serde(borrow)]
    pub approvals: Vec<Approval<'a>>,
    // pub size_insertions: Option<i64>,
    // pub size_deletions: Option<i64>,
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
pub struct RefUpdate<'a> {
    #[serde(borrow)]
    pub old_rev: &'a str,
    /// All-zero SHA indicates the ref was deleted.
    #[serde(borrow)]
    pub new_rev: &'a str,
    #[serde(borrow)]
    pub ref_name: &'a str,
    #[serde(borrow)]
    pub project: &'a str,
}
