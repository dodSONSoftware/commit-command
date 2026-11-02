//! An upstream configuration change has independent, exact native authority.
use super::observation::ObservationPlan;
use crate::domain::{Issue, ObjectId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConfigBinding {
    pub identity: String,
    pub digest: String,
    pub mode: u32,
    pub group: u32,
    pub attributes_digest: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpstreamPlan {
    #[serde(flatten)]
    pub transport: ObservationPlan,
    pub source_ref: String,
    pub source_oid: ObjectId,
    pub observation_id: String,
    pub observed_oid: ObjectId,
    pub observed_at_ms: u64,
    pub expected_config: ConfigBinding,
    pub candidate_digest: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UpstreamPhase {
    Proposed,
    LockPending,
    LockCreated,
    LockReady,
    PublicationPending,
    ConfigPublished,
    Complete,
    Blocked,
    Cancelled,
    Interrupted,
    Uncertain,
    Aborted,
    ObservedConfiguration,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UpstreamStatus {
    Unchanged,
    Updated,
    MatchesTarget,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpstreamOperation {
    pub plan: UpstreamPlan,
    pub phase: UpstreamPhase,
    pub configuration: UpstreamStatus,
    pub issue: Option<Issue>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpstreamRecovery {
    pub operation: UpstreamOperation,
    pub action: String,
    pub fingerprint: String,
    pub explanation: String,
}
