use serde::{Deserialize, Serialize};

pub type SenseId = String;
pub type ActId = String;
pub type CycleId = u64;

pub fn is_uuid_v4(id: &str) -> bool {
    uuid::Uuid::parse_str(id)
        .map(|uuid| uuid.get_version_num() == 4)
        .unwrap_or(false)
}

pub fn is_uuid_v7(id: &str) -> bool {
    uuid::Uuid::parse_str(id)
        .map(|uuid| uuid.get_version_num() == 7)
        .unwrap_or(false)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum NeuralSignalType {
    Sense,
    Act,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub struct NeuralSignalDescriptorRouteKey {
    #[serde(rename = "type")]
    pub r#type: NeuralSignalType,
    pub endpoint_id: String,
    pub neural_signal_descriptor_id: String,
}

impl NeuralSignalDescriptorRouteKey {
    pub fn fq_neural_signal_descriptor_id(&self) -> String {
        format!(
            "{}/{}/{}",
            match self.r#type {
                NeuralSignalType::Sense => "sense",
                NeuralSignalType::Act => "act",
            },
            self.endpoint_id,
            self.neural_signal_descriptor_id
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NeuralSignalDescriptor {
    #[serde(rename = "type")]
    pub r#type: NeuralSignalType,
    pub endpoint_id: String,
    pub neural_signal_descriptor_id: String,
    pub payload_schema: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct NeuralSignalDescriptorCatalog {
    pub version: String,
    #[serde(default)]
    pub entries: Vec<NeuralSignalDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct NeuralSignalDescriptorPatch {
    #[serde(default)]
    pub entries: Vec<NeuralSignalDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct NeuralSignalDescriptorDropPatch {
    #[serde(default)]
    pub routes: Vec<NeuralSignalDescriptorRouteKey>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SenseDatum {
    pub sense_id: SenseId,
    pub endpoint_id: String,
    pub neural_signal_descriptor_id: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Sense {
    Domain(SenseDatum),
    Hibernate,
    NewNeuralSignalDescriptors(NeuralSignalDescriptorPatch),
    DropNeuralSignalDescriptors(NeuralSignalDescriptorDropPatch),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Act {
    pub act_id: ActId,
    pub endpoint_id: String,
    pub neural_signal_descriptor_id: String,
    pub payload: serde_json::Value,
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
    pub capabilities: NeuralSignalDescriptorCatalog,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchDecision {
    Continue,
    Break,
}
