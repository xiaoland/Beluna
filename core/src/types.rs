use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub type SenseInstanceId = String;
pub type ActInstanceId = String;
pub type CycleId = u64;

pub fn default_sense_weight() -> f64 {
    0.0
}

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
    pub fn fq_neural_signal_id(&self) -> String {
        build_fq_neural_signal_id(&self.endpoint_id, &self.neural_signal_descriptor_id)
    }
}

pub fn build_fq_neural_signal_id(endpoint_id: &str, neural_signal_descriptor_id: &str) -> String {
    format!("{endpoint_id}/{neural_signal_descriptor_id}")
}

pub fn is_valid_neural_signal_identifier(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }

    let mut in_segment = false;
    for ch in value.chars() {
        if ch == '.' {
            if !in_segment {
                return false;
            }
            in_segment = false;
            continue;
        }
        if !ch.is_ascii_alphanumeric() && ch != '-' {
            return false;
        }
        in_segment = true;
    }
    in_segment
}

#[cfg(test)]
mod tests {
    use super::is_valid_neural_signal_identifier;

    #[test]
    fn accepts_ascii_alnum_dot_and_dash() {
        for value in ["abc", "a1.b2", "aa-bb", "aa.bb-cc", "aa-bb.cc-dd"] {
            assert!(is_valid_neural_signal_identifier(value), "{value}");
        }
    }

    #[test]
    fn rejects_empty_invalid_or_malformed_identifier() {
        for value in ["", ".aa", "aa.", "aa..bb", "aa_bb", "aa/bb"] {
            assert!(!is_valid_neural_signal_identifier(value), "{value}");
        }
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeuralSignalDescriptorPatchRejection {
    pub entry: NeuralSignalDescriptorRouteKey,
    pub reason_code: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct NeuralSignalDescriptorPatchCommit {
    #[serde(default)]
    pub accepted_entries: Vec<NeuralSignalDescriptor>,
    #[serde(default)]
    pub rejected_entries: Vec<NeuralSignalDescriptorPatchRejection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct NeuralSignalDescriptorDropPatch {
    #[serde(default)]
    pub routes: Vec<NeuralSignalDescriptorRouteKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeuralSignalDescriptorDropRejection {
    pub route: NeuralSignalDescriptorRouteKey,
    pub reason_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct NeuralSignalDescriptorDropCommit {
    #[serde(default)]
    pub accepted_routes: Vec<NeuralSignalDescriptorRouteKey>,
    #[serde(default)]
    pub rejected_routes: Vec<NeuralSignalDescriptorDropRejection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProprioceptionPatch {
    #[serde(default)]
    pub entries: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProprioceptionDropPatch {
    #[serde(default)]
    pub keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sense {
    pub sense_instance_id: SenseInstanceId,
    pub endpoint_id: String,
    pub neural_signal_descriptor_id: String,
    pub payload: String,
    #[serde(default = "default_sense_weight")]
    pub weight: f64,
    #[serde(default)]
    pub act_instance_id: Option<ActInstanceId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Act {
    pub act_instance_id: ActInstanceId,
    pub endpoint_id: String,
    pub neural_signal_descriptor_id: String,
    #[serde(default)]
    pub might_emit_sense_ids: Vec<String>,
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
    pub ns_descriptor: NeuralSignalDescriptorCatalog,
    #[serde(default)]
    pub proprioception: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchDecision {
    Continue,
    Break,
}
