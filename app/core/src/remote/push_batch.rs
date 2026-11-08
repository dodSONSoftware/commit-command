//! A durable guided queue is planning data; every publication needs a new native approval.
use super::push::{PushDependencyPlan, PushPlanningSelection};
use crate::{
    domain::{Issue, WorkspaceId, WorktreeId},
    writes::HookPolicy,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PushBatchSelection {
    pub selection: PushPlanningSelection,
    pub hooks: HookPolicy,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PushBatchPlan {
    pub id: String,
    pub workspace: WorkspaceId,
    pub created_at_ms: u64,
    pub selections: Vec<PushBatchSelection>,
    pub dependency_plan: PushDependencyPlan,
    pub digest: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PushBatchBinding {
    pub id: String,
    pub digest: String,
    pub worktree: WorktreeId,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PushBatchPhase {
    Ready,
    Executing,
    Complete,
    Stopped,
    Interrupted,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PushBatchItemPhase {
    Pending,
    SkippedObservedCurrent,
    ReviewPending,
    Executing,
    Complete,
    Blocked,
    Cancelled,
    Unconfirmed,
    Interrupted,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PushBatchProgress {
    pub worktree: WorktreeId,
    pub phase: PushBatchItemPhase,
    pub publication: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PushBatch {
    pub plan: PushBatchPlan,
    pub phase: PushBatchPhase,
    pub items: Vec<PushBatchProgress>,
    pub issue: Option<Issue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PushBatchPage {
    pub items: Vec<PushBatch>,
    pub next: Option<usize>,
}
