use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub type ActionId = String;
pub type AttemptId = String;
pub type ReserveEntryId = String;
pub type CostAttributionId = String;
pub type CycleId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpineExecutionMode {
    BestEffortReplayable,
    SerializedDeterministic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CostVector {
    pub survival_micro: i64,
    pub time_ms: u64,
    pub io_units: u64,
    pub token_units: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RouteKey {
    pub affordance_key: String,
    pub capability_handle: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EndpointCapabilityDescriptor {
    pub route: RouteKey,
    pub payload_schema: serde_json::Value,
    pub max_payload_bytes: usize,
    #[serde(default)]
    pub default_cost: CostVector,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EndpointRegistration {
    pub endpoint_id: String,
    pub descriptor: EndpointCapabilityDescriptor,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SpineCapabilityCatalog {
    pub version: u64,
    #[serde(default)]
    pub entries: Vec<EndpointCapabilityDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdmittedAction {
    pub action_id: ActionId,
    pub source_attempt_id: AttemptId,
    pub reserve_entry_id: ReserveEntryId,
    pub cost_attribution_id: CostAttributionId,
    pub affordance_key: String,
    pub capability_handle: String,
    pub normalized_payload: serde_json::Value,
    pub reserved_cost: CostVector,
    pub degraded: bool,
    #[serde(default)]
    pub degradation_profile_id: Option<String>,
    pub admission_cycle: CycleId,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdmittedActionBatch {
    pub cycle_id: CycleId,
    pub actions: Vec<AdmittedAction>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EndpointInvocation {
    pub action: AdmittedAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EndpointExecutionOutcome {
    Applied {
        actual_cost_micro: i64,
        reference_id: String,
    },
    Rejected {
        reason_code: String,
        reference_id: String,
    },
    Deferred {
        reason_code: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SpineEvent {
    ActionApplied {
        action_id: ActionId,
        reserve_entry_id: ReserveEntryId,
        cost_attribution_id: CostAttributionId,
        actual_cost_micro: i64,
        reference_id: String,
    },
    ActionRejected {
        action_id: ActionId,
        reserve_entry_id: ReserveEntryId,
        cost_attribution_id: CostAttributionId,
        reason_code: String,
        reference_id: String,
    },
    ActionDeferred {
        action_id: ActionId,
        reason_code: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderedSpineEvent {
    pub seq_no: u64,
    pub event: SpineEvent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpineExecutionReport {
    pub mode: SpineExecutionMode,
    pub events: Vec<OrderedSpineEvent>,
    #[serde(default)]
    pub replay_cursor: Option<String>,
}
