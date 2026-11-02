//! Serializable plans/results contain no credentials, raw configuration or execution capabilities.
use super::*;
use crate::{domain::Issue, inspection::Metadata};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ToolBinding {
    pub role: String,
    pub path: NativePath,
    pub fingerprint: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ObservationPlan {
    pub id: String,
    pub workspace: WorkspaceId,
    pub worktree: WorktreeId,
    pub metadata: Metadata,
    pub remote: String,
    pub reference: String,
    pub destination: String,
    pub transport: Transport,
    pub config_digest: String,
    pub generation: String,
    pub created_at_ms: u64,
    pub tools: Vec<ToolBinding>,
    pub local_target: Option<NativePath>,
    pub local_binding: Option<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ObservationPhase {
    Proposed,
    Running,
    Complete,
    Blocked,
    Cancelled,
    Interrupted,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Observation {
    pub plan: ObservationPlan,
    pub phase: ObservationPhase,
    pub oid: Option<ObjectId>,
    pub observed_at_ms: Option<u64>,
    pub issue: Option<Issue>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ObservationPage {
    pub items: Vec<Observation>,
    pub next: Option<usize>,
}
