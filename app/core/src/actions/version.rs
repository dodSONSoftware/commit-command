//! Durable version-file invocation data, never an execution capability.
use super::semver::{ObservedVersionState, VersionPreview};
use crate::{domain::*, inspection::Metadata};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FileStamp {
    pub device: u64,
    pub inode: u64,
    pub mode: u32,
    pub length: u64,
    pub modified_seconds: i64,
    pub modified_nanos: i64,
    pub changed_seconds: i64,
    pub changed_nanos: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VersionPlan {
    pub id: String,
    pub workspace: WorkspaceId,
    pub worktree: WorktreeId,
    pub metadata: Metadata,
    pub path: NativePath,
    pub version: VersionPreview,
    pub file: FileStamp,
    pub root_device: u64,
    pub root_inode: u64,
    pub parent_device: u64,
    pub parent_inode: u64,
    pub config_digest: String,
    pub index_digest: String,
    pub branch: String,
    pub head: Option<String>,
    pub created_at_ms: u64,
    pub digest: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VersionPhase {
    Proposed,
    Preparing,
    Replacing,
    Applied,
    Blocked,
    Cancelled,
    Unconfirmed,
    Interrupted,
    Reconciled,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VersionInvocation {
    pub plan: VersionPlan,
    pub phase: VersionPhase,
    pub definition_approved_at_ms: Option<u64>,
    pub candidate: Option<FileStamp>,
    pub applied: bool,
    pub observed: Option<ObservedVersionState>,
    pub finished_at_ms: Option<u64>,
    pub issue: Option<Issue>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VersionPage {
    pub items: Vec<VersionInvocation>,
    pub next: Option<usize>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VersionRecovery {
    pub invocation: VersionInvocation,
    pub observed: ObservedVersionState,
    pub file: FileStamp,
    pub backup_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RetainedVersionFile {
    pub operation: String,
    pub location: NativePath,
    pub bytes: Vec<u8>,
    pub fingerprint: String,
    pub matches_recorded_before: bool,
    pub read_at_ms: u64,
}
