use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::admission::types::{AdmissionReport, IntentAttempt};

pub type GoalId = String;
pub type CommitmentId = String;
pub type CycleId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalClass {
    Primary,
    Supporting,
    Exploratory,
    Maintenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalScope {
    Strategic,
    Tactical,
    Session,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GoalMetadataValue {
    Text(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Json(serde_json::Value),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataSource {
    User,
    Cortex,
    Runtime,
    Imported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetadataProvenance {
    pub source: MetadataSource,
    pub recorded_cycle: CycleId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoalMetadataEntry {
    pub value: GoalMetadataValue,
    pub provenance: MetadataProvenance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Goal {
    pub id: GoalId,
    pub title: String,
    pub class: GoalClass,
    pub scope: GoalScope,
    #[serde(default)]
    pub parent_goal_id: Option<GoalId>,
    #[serde(default)]
    pub metadata: BTreeMap<String, GoalMetadataEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommitmentStatus {
    Proposed,
    Active,
    Paused,
    Cancelled,
    Completed,
    Failed,
}

impl CommitmentStatus {
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Cancelled | Self::Completed | Self::Failed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitmentRecord {
    pub commitment_id: CommitmentId,
    pub goal_id: GoalId,
    pub status: CommitmentStatus,
    pub created_cycle: CycleId,
    pub last_transition_cycle: CycleId,
    #[serde(default)]
    pub superseded_by: Option<CommitmentId>,
    #[serde(default)]
    pub failure_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchedulingContext {
    pub commitment_id: CommitmentId,
    pub cycle_id: CycleId,
    pub dynamic_priority: u16,
    pub queue_position: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CortexCommand {
    ProposeGoal(Goal),
    CommitGoal {
        goal_id: GoalId,
        #[serde(default)]
        commitment_id: Option<CommitmentId>,
    },
    SetCommitmentStatus {
        commitment_id: CommitmentId,
        status: CommitmentStatus,
        #[serde(default)]
        superseded_by: Option<CommitmentId>,
        #[serde(default)]
        failure_code: Option<String>,
    },
    ObserveAdmissionReport(AdmissionReport),
    PlanNow,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CortexEvent {
    GoalRegistered {
        goal_id: GoalId,
    },
    CommitmentCreated {
        commitment_id: CommitmentId,
        goal_id: GoalId,
    },
    CommitmentStatusChanged {
        commitment_id: CommitmentId,
        status: CommitmentStatus,
    },
    AdmissionObserved {
        cycle_id: CycleId,
        outcomes: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CortexCycleOutput {
    pub cycle_id: CycleId,
    pub events: Vec<CortexEvent>,
    pub scheduling: Vec<SchedulingContext>,
    pub attempts: Vec<IntentAttempt>,
}
