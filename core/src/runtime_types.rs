use serde::{Deserialize, Serialize};

pub use crate::cortex::{
    Act, ActId, CapabilityDropPatch, CapabilityPatch, RequestedResources, Sense, SenseDatum,
    SenseId,
};

pub type CycleId = u64;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchDecision {
    Continue,
    Break,
}
