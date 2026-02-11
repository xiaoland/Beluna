use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::{
    admission::types::{AttributionRecord, CostAttributionId},
    continuity::types::{SenseSample, SituationView},
    ledger::{
        SurvivalLedger,
        types::{CycleId, PolicyVersionTuple},
    },
};

const MAX_RECENT_SENSE: usize = 128;

#[derive(Debug, Clone)]
pub struct ContinuityState {
    pub cycle_id: CycleId,
    pub ledger: SurvivalLedger,
    pub attribution_index: BTreeMap<CostAttributionId, Vec<AttributionRecord>>,
    pub seen_external_reference_ids: BTreeSet<String>,
    pub affordance_registry_version: String,
    pub cost_policy_version: String,
    pub admission_ruleset_version: String,
    pub recent_sense: VecDeque<SenseSample>,
}

impl ContinuityState {
    pub fn new(initial_survival_micro: i64) -> Self {
        Self {
            cycle_id: 0,
            ledger: SurvivalLedger::new(initial_survival_micro),
            attribution_index: BTreeMap::new(),
            seen_external_reference_ids: BTreeSet::new(),
            affordance_registry_version: "v1".to_string(),
            cost_policy_version: "v1".to_string(),
            admission_ruleset_version: "v1".to_string(),
            recent_sense: VecDeque::new(),
        }
    }

    pub fn version_tuple(&self) -> PolicyVersionTuple {
        PolicyVersionTuple {
            affordance_registry_version: self.affordance_registry_version.clone(),
            cost_policy_version: self.cost_policy_version.clone(),
            admission_ruleset_version: self.admission_ruleset_version.clone(),
        }
    }

    pub fn ingest_sense(&mut self, sample: SenseSample) {
        self.recent_sense.push_back(sample);
        while self.recent_sense.len() > MAX_RECENT_SENSE {
            self.recent_sense.pop_front();
        }
    }

    pub fn build_situation(&self) -> SituationView {
        SituationView {
            cycle_id: self.cycle_id,
            available_survival_micro: self.ledger.available_survival_micro(),
            open_reservation_count: self
                .ledger
                .reservations
                .values()
                .filter(|reservation| {
                    matches!(
                        reservation.state,
                        crate::ledger::types::ReservationState::Open
                    )
                })
                .count(),
            recent_sense_count: self.recent_sense.len(),
        }
    }
}

impl Default for ContinuityState {
    fn default() -> Self {
        Self::new(1_000_000)
    }
}
