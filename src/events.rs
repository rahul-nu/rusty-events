use crate::approval::{Account, Approval};
use crate::model::{Change, ChangeKey, PatchSet, RefUpdate, Status};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GerritEventType {
    #[serde(rename = "change-abandoned")]
    #[serde(rename_all = "camelCase")]
    ChangeAbandoned {
        change: Change,
        patch_set: PatchSet,
        abandoner: Account,
        reason: String,
        event_created_on: i64,
    },
    #[serde(rename = "change-deleted")]
    #[serde(rename_all = "camelCase")]
    ChangeDeleted { change: Change, deleter: Account },
    #[serde(rename = "change-merged")]
    #[serde(rename_all = "camelCase")]
    ChangeMerged {
        change: Change,
        patch_set: PatchSet,
        submitter: Account,
        new_rev: String,
        event_created_on: i64,
    },
    #[serde(rename = "change-restored")]
    #[serde(rename_all = "camelCase")]
    ChangeRestored {
        change: Change,
        patch_set: PatchSet,
        restorer: Account,
        reason: String,
        event_created_on: i64,
    },
    #[serde(rename = "dropped-output")]
    #[serde(rename_all = "camelCase")]
    DroppedOutput,
    #[serde(rename = "comment-added")]
    #[serde(rename_all = "camelCase")]
    CommentAdded {
        author: Account,
        approvals: Vec<Approval>,
        comment: Option<String>,
        patch_set: PatchSet,
        change: Change,
        event_created_on: i64,
        instance_id: String,
    },
    #[serde(rename = "patchset-created")]
    #[serde(rename_all = "camelCase")]
    PatchsetCreated {
        uploader: Account,
        change: Change,
        patch_set: PatchSet,
        event_created_on: i64,
    },
    #[serde(rename = "fetch-ref-replicated")]
    #[serde(rename_all = "camelCase")]
    FetchRefReplicated {
        ref_update_result: Status,
        project: String,
        #[serde(rename = "ref")]
        ref_name: String,
        status: String,
        target_uri: String,
        event_created_on: i64,
        instance_id: String,
    },
    #[serde(rename = "fetch-ref-replication-scheduled")]
    #[serde(rename_all = "camelCase")]
    FetchRefReplicationScheduled {
        project: String,
        target_uri: String,
        event_created_on: i64,
        instance_id: String,
    },
    #[serde(rename = "ref-replication-scheduled")]
    #[serde(rename_all = "camelCase")]
    RefReplicationScheduled {
        event_created_on: i64,
        instance_id: String,
        target_uri: String,
        #[serde(rename = "ref")]
        ref_name: String,
        project: String,
    },
    #[serde(rename = "hashtags-changed")]
    #[serde(rename_all = "camelCase")]
    HashtagsChanged {
        change: Change,
        editor: Account,
        #[serde(default)]
        added: Vec<String>,
        #[serde(default)]
        removed: Vec<String>,
        #[serde(default)]
        hashtags: Vec<String>,
        event_created_on: i64,
    },
    #[serde(rename = "project-created")]
    #[serde(rename_all = "camelCase")]
    ProjectCreated {
        project_name: String,
        project_head: String,
        event_created_on: i64,
    },
    #[serde(rename = "ref-updated")]
    #[serde(rename_all = "camelCase")]
    RefUpdated {
        submitter: Option<Account>,
        ref_update: RefUpdate,
        event_created_on: i64,
    },
    #[serde(rename = "reviewer-added")]
    #[serde(rename_all = "camelCase")]
    ReviewerAdded {
        change: Change,
        patch_set: PatchSet,
        reviewer: Account,
        adder: Account,
        event_created_on: i64,
    },
    #[serde(rename = "reviewer-deleted")]
    #[serde(rename_all = "camelCase")]
    ReviewerDeleted {
        change: Change,
        patch_set: PatchSet,
        reviewer: Account,
        remover: Account,
        #[serde(default)]
        approvals: Vec<Approval>,
        comment: Option<String>,
        event_created_on: i64,
    },
    #[serde(rename = "topic-changed")]
    #[serde(rename_all = "camelCase")]
    TopicChanged {
        change: Change,
        changer: Account,
        old_topic: Option<String>,
        event_created_on: i64,
    },
    #[serde(rename = "batch-ref-updated")]
    #[serde(rename_all = "camelCase")]
    BatchRefUpdated {
        submitter: Option<Account>,
        ref_updates: Vec<RefUpdate>,
        event_created_on: i64,
    },
    #[serde(rename = "wip-state-changed")]
    #[serde(rename_all = "camelCase")]
    WipStateChanged {
        change: Change,
        patch_set: PatchSet,
        changer: Account,
        event_created_on: i64,
    },
    #[serde(rename = "private-state-changed")]
    #[serde(rename_all = "camelCase")]
    PrivateStateChanged {
        change: Change,
        patch_set: PatchSet,
        changer: Account,
        event_created_on: i64,
        change_key: ChangeKey,
    },
    #[serde(rename = "vote-deleted")]
    #[serde(rename_all = "camelCase")]
    VoteDeleted {
        change: Change,
        patch_set: PatchSet,
        reviewer: Account,
        remover: Account,
        #[serde(default)]
        approvals: Vec<Approval>,
        comment: Option<String>,
    },
    #[serde(rename = "project-head-updated")]
    #[serde(rename_all = "camelCase")]
    ProjectHeadUpdated {
        old_head: String,
        new_head: String,
        event_created_on: i64,
    },
}
