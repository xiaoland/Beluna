use serde::{Deserialize, Serialize};

pub type SenseId = String;
pub type ActId = String;
pub type CycleId = u64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RequestedResources {
    pub survival_micro: i64,
    pub time_ms: u64,
    pub io_units: u64,
    pub token_units: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SenseDatum {
    pub sense_id: SenseId,
    pub source: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CapabilityPatch {
    #[serde(default)]
    pub entries: Vec<crate::spine::types::EndpointCapabilityDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CapabilityDropPatch {
    #[serde(default)]
    pub routes: Vec<crate::spine::types::RouteKey>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Sense {
    Domain(SenseDatum),
    Sleep,
    NewCapabilities(CapabilityPatch),
    DropCapabilities(CapabilityDropPatch),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoalFrame {
    pub goal_id: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CognitionState {
    pub revision: u64,
    #[serde(default)]
    pub goal_stack: Vec<GoalFrame>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PhysicalLedgerSnapshot {
    pub available_survival_micro: i64,
    pub open_reservation_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhysicalState {
    pub cycle_id: CycleId,
    pub ledger: PhysicalLedgerSnapshot,
    pub capabilities: crate::cortex::CapabilityCatalog,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Act {
    pub act_id: ActId,
    #[serde(default)]
    pub based_on: Vec<SenseId>,
    pub endpoint_id: String,
    pub capability_id: String,
    pub capability_instance_id: String,
    pub normalized_payload: serde_json::Value,
    #[serde(default)]
    pub requested_resources: RequestedResources,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchDecision {
    Continue,
    Break,
}
