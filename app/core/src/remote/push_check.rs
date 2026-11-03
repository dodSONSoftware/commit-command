//! Separately approved hook checks. A completed check is never publication authority.
use super::{
    observation::ObservationPlan,
    push::{PushDependency, PushPlanningSelection, PushPreview},
};
use crate::domain::{ObjectId, WorkspaceId, WorktreeId};
use crate::writes::{Hook, HookPolicy};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PushCheckPlan {
    pub id: String,
    pub workspace: WorkspaceId,
    pub worktree: WorktreeId,
    pub created_at_ms: u64,
    pub selection: PushPlanningSelection,
    pub preview: PushPreview,
    pub dependencies: Vec<PushDependency>,
    pub transport: ObservationPlan,
    pub hooks: HookPolicy,
    pub hook: Option<Hook>,
    pub hook_args: [String; 2],
    pub hook_stdin: String,
    pub digest: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PushCheckPhase {
    Proposed,
    Running,
    Complete,
    Blocked,
    Cancelled,
    Interrupted,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PushCheck {
    pub plan: PushCheckPlan,
    pub phase: PushCheckPhase,
    pub checked_at_ms: Option<u64>,
    pub source_oid: ObjectId,
    pub hook_passed: bool,
    pub published: bool,
    pub issue: Option<String>,
}
