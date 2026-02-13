use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::{
    admission::types::{AdmissionReport, AttributionRecord, CostAttributionId, IntentAttempt},
    cortex::SenseDelta,
    ledger::types::CycleId,
    spine::types::{ActionId, SpineExecutionReport},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalDebitObservation {
    pub reference_id: String,
    pub cost_attribution_id: CostAttributionId,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    #[serde(default)]
    pub cycle_id: Option<CycleId>,
    pub debit_survival_micro: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SenseSample {
    pub source: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SituationView {
    pub cycle_id: CycleId,
    pub available_survival_micro: i64,
    pub open_reservation_count: usize,
    pub recent_sense_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContinuityCycleOutput {
    pub cycle_id: CycleId,
    pub admission_report: AdmissionReport,
    pub spine_report: SpineExecutionReport,
    pub admitted_action_count: usize,
    pub external_debit_applied_count: usize,
    pub expired_reservation_count: usize,
}

#[derive(Debug, Clone, Default)]
pub struct AttributionJournal(
    pub std::collections::BTreeMap<CostAttributionId, Vec<AttributionRecord>>,
);

#[derive(Debug, Clone, Default)]
pub struct RecentSense(pub VecDeque<SenseSample>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NeuralSignalBatch {
    pub reaction_id: u64,
    #[serde(default)]
    pub attempts: Vec<IntentAttempt>,
}

#[derive(Debug, Clone, Default)]
pub struct SenseQueue(pub VecDeque<SenseDelta>);

#[derive(Debug, Clone, Default)]
pub struct NeuralSignalQueue(pub VecDeque<NeuralSignalBatch>);
