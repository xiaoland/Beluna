use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

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
    pub endpoint_id: String,
    pub capability_id: String,
}

impl RouteKey {
    pub fn fq_capability_id(&self) -> String {
        format!("{}/{}", self.endpoint_id, self.capability_id)
    }
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SpineCapabilityCatalog {
    pub version: u64,
    #[serde(default)]
    pub entries: Vec<EndpointCapabilityDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActDispatchResult {
    Acknowledged {
        reference_id: String,
    },
    Rejected {
        reason_code: String,
        reference_id: String,
    },
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
        reference_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SpineEvent {
    ActApplied {
        cycle_id: CycleId,
        seq_no: u64,
        act_id: String,
        capability_instance_id: String,
        reserve_entry_id: ReserveEntryId,
        cost_attribution_id: CostAttributionId,
        actual_cost_micro: i64,
        reference_id: String,
    },
    ActRejected {
        cycle_id: CycleId,
        seq_no: u64,
        act_id: String,
        capability_instance_id: String,
        reserve_entry_id: ReserveEntryId,
        cost_attribution_id: CostAttributionId,
        reason_code: String,
        reference_id: String,
    },
    ActDeferred {
        cycle_id: CycleId,
        seq_no: u64,
        act_id: String,
        capability_instance_id: String,
        reserve_entry_id: ReserveEntryId,
        cost_attribution_id: CostAttributionId,
        reason_code: String,
        reference_id: String,
    },
}

impl SpineEvent {
    pub fn reserve_entry_id(&self) -> &str {
        match self {
            SpineEvent::ActApplied {
                reserve_entry_id, ..
            }
            | SpineEvent::ActRejected {
                reserve_entry_id, ..
            }
            | SpineEvent::ActDeferred {
                reserve_entry_id, ..
            } => reserve_entry_id,
        }
    }

    pub fn reference_id(&self) -> &str {
        match self {
            SpineEvent::ActApplied { reference_id, .. }
            | SpineEvent::ActRejected { reference_id, .. }
            | SpineEvent::ActDeferred { reference_id, .. } => reference_id,
        }
    }

    pub fn cost_attribution_id(&self) -> &str {
        match self {
            SpineEvent::ActApplied {
                cost_attribution_id,
                ..
            }
            | SpineEvent::ActRejected {
                cost_attribution_id,
                ..
            }
            | SpineEvent::ActDeferred {
                cost_attribution_id,
                ..
            } => cost_attribution_id,
        }
    }
}
