//! Exact publication plans are separate from hook checks and never replay after interruption.
use super::{observation::Observation, push_check::PushCheckPlan};
use crate::domain::{Issue, ObjectId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PushPublicationPlan {
    pub id: String,
    pub created_at_ms: u64,
    pub check: PushCheckPlan,
    pub digest: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PushPublicationPhase {
    Proposed,
    Checking,
    Preparing,
    Publishing,
    Complete,
    Blocked,
    Cancelled,
    Interrupted,
    Unconfirmed,
    Reconciled,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PushRemoteState {
    ApprovedSourcePresent,
    ExpectedOldPresent,
    Different,
    Missing,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PushPublication {
    pub plan: PushPublicationPlan,
    pub phase: PushPublicationPhase,
    pub published: bool,
    pub may_have_published: bool,
    pub finished_at_ms: Option<u64>,
    pub observed_remote_oid: Option<ObjectId>,
    pub remote_state: Option<PushRemoteState>,
    pub reconciled_observation: Option<String>,
    pub object_count: Option<u32>,
    pub object_bytes: Option<u64>,
    pub issue: Option<Issue>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PushPublicationRecovery {
    pub operation: PushPublication,
    pub observation: Observation,
    pub remote_state: PushRemoteState,
}
