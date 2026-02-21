use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalDebitObservation {
    pub reference_id: String,
    pub cost_attribution_id: String,
    #[serde(default)]
    pub action_id: Option<String>,
    #[serde(default)]
    pub cycle_id: Option<u64>,
    pub debit_survival_micro: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchContext {
    pub cycle_id: u64,
    pub act_seq_no: u64,
}
