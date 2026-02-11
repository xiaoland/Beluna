use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::spine::types::ActionId;

pub type AttemptId = String;
pub type GoalId = String;
pub type CommitmentId = String;
pub type CostAttributionId = String;
pub type CycleId = u64;
pub type ConstraintCode = String;
pub type EconomicCode = String;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RequestedResources {
    pub survival_micro: i64,
    pub time_ms: u64,
    pub io_units: u64,
    pub token_units: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntentAttempt {
    pub attempt_id: AttemptId,
    pub cycle_id: CycleId,
    pub commitment_id: CommitmentId,
    pub goal_id: GoalId,
    pub planner_slot: u16,
    pub affordance_key: String,
    pub capability_handle: String,
    pub normalized_payload: serde_json::Value,
    pub requested_resources: RequestedResources,
    pub cost_attribution_id: CostAttributionId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AffordabilitySnapshot {
    pub available_survival_micro: i64,
    pub required_survival_micro: i64,
    pub required_time_ms: u64,
    pub required_io_units: u64,
    pub required_token_units: u64,
    pub max_time_ms: u64,
    pub max_io_units: u64,
    pub max_token_units: u64,
}

impl AffordabilitySnapshot {
    pub fn survival_affordable(&self) -> bool {
        self.available_survival_micro >= self.required_survival_micro
    }

    pub fn within_runtime_limits(&self) -> bool {
        self.required_time_ms <= self.max_time_ms
            && self.required_io_units <= self.max_io_units
            && self.required_token_units <= self.max_token_units
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AdmissionDisposition {
    Admitted { degraded: bool },
    DeniedHard { code: ConstraintCode },
    DeniedEconomic { code: EconomicCode },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AdmissionWhy {
    HardRule {
        code: ConstraintCode,
    },
    Economic {
        code: EconomicCode,
        available_survival_micro: i64,
        required_survival_micro: i64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReservationDelta {
    pub reserve_entry_id: String,
    pub reserved_survival_micro: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdmissionReportItem {
    pub attempt_id: AttemptId,
    pub disposition: AdmissionDisposition,
    #[serde(default)]
    pub why: Option<AdmissionWhy>,
    #[serde(default)]
    pub ledger_delta: Option<ReservationDelta>,
    #[serde(default)]
    pub admitted_action_id: Option<ActionId>,
    #[serde(default)]
    pub degradation_profile_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AdmissionReport {
    pub cycle_id: CycleId,
    pub outcomes: Vec<AdmissionReportItem>,
    pub total_reserved_survival_micro: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributionRecord {
    pub action_id: ActionId,
    pub reserve_entry_id: String,
    pub cycle_id: CycleId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetadataValue {
    Text(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Json(serde_json::Value),
}

#[derive(Debug, Clone, Default)]
pub struct AdmissionAttributionIndex(pub BTreeMap<CostAttributionId, Vec<AttributionRecord>>);
