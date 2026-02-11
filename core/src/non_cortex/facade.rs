use std::sync::Arc;

use crate::{
    non_cortex::{
        affordance::{AffordanceProfile, AffordanceRegistry},
        continuity::assert_settlement_consistency,
        debit_sources::ExternalDebitSourcePort,
        error::{NonCortexError, invariant_violation},
        ports::SpinePort,
        resolver::{AdmissionResolver, AdmissionResolverConfig, CostAdmissionPolicy},
        types::{ExternalDebitObservation, IntentAttempt, NonCortexCycleOutput, NonCortexState},
    },
    spine::{noop::DeterministicNoopSpine, types::SpineEvent},
};

pub struct NonCortexFacade {
    state: NonCortexState,
    resolver: AdmissionResolver,
    spine: Arc<dyn SpinePort>,
    external_debit_source: Arc<dyn ExternalDebitSourcePort>,
}

impl NonCortexFacade {
    pub fn new(
        state: NonCortexState,
        resolver: AdmissionResolver,
        spine: Arc<dyn SpinePort>,
        external_debit_source: Arc<dyn ExternalDebitSourcePort>,
    ) -> Self {
        Self {
            state,
            resolver,
            spine,
            external_debit_source,
        }
    }

    pub fn with_defaults(initial_survival_micro: i64) -> Self {
        let default_registry = AffordanceRegistry::new(vec![
            AffordanceProfile::default(),
            AffordanceProfile {
                profile_id: "default-execution".to_string(),
                affordance_key: "execute.tool".to_string(),
                capability_handle: "cap.core".to_string(),
                max_payload_bytes: 16_384,
                base_cost: crate::spine::types::CostVector {
                    survival_micro: 400,
                    time_ms: 200,
                    io_units: 2,
                    token_units: 256,
                },
                degradation_profiles: vec![],
            },
        ]);

        let resolver = AdmissionResolver::new(
            default_registry,
            CostAdmissionPolicy::default(),
            AdmissionResolverConfig::default(),
        );

        Self::new(
            NonCortexState::new(initial_survival_micro),
            resolver,
            Arc::new(crate::non_cortex::noop::SpinePortAdapter::new(Arc::new(
                DeterministicNoopSpine::default(),
            ))),
            Arc::new(crate::non_cortex::noop::NoopDebitSource),
        )
    }

    pub fn state(&self) -> &NonCortexState {
        &self.state
    }

    pub fn process_attempts(
        &mut self,
        cycle_id: u64,
        attempts: Vec<IntentAttempt>,
    ) -> Result<NonCortexCycleOutput, NonCortexError> {
        self.state.cycle_id = cycle_id;

        let (admission_report, admitted_batch) =
            self.resolver
                .admit_attempts(&mut self.state, cycle_id, attempts)?;

        let admitted_action_count = admitted_batch.actions.len();
        let spine_report = self.spine.execute_admitted(admitted_batch)?;

        self.reconcile_spine_report(cycle_id, &spine_report)?;

        let expired_reservations = self.state.ledger.expire_open_reservations(
            cycle_id,
            "expiry",
            self.state.version_tuple(),
        )?;

        let external_debit_applied_count = self.apply_external_debits(cycle_id)?;

        assert_settlement_consistency(&self.state)?;

        Ok(NonCortexCycleOutput {
            cycle_id,
            admission_report,
            spine_report,
            admitted_action_count,
            external_debit_applied_count,
            expired_reservation_count: expired_reservations.len(),
        })
    }

    fn reconcile_spine_report(
        &mut self,
        cycle_id: u64,
        spine_report: &crate::spine::types::SpineExecutionReport,
    ) -> Result<(), NonCortexError> {
        let mut ordered_events = spine_report.events.clone();
        ordered_events.sort_by_key(|event| event.seq_no);

        for window in ordered_events.windows(2) {
            if window[0].seq_no == window[1].seq_no {
                return Err(invariant_violation(format!(
                    "duplicate spine seq_no '{}' during reconciliation",
                    window[0].seq_no
                )));
            }
        }

        for ordered_event in ordered_events {
            match ordered_event.event {
                SpineEvent::ActionApplied {
                    action_id,
                    reserve_entry_id,
                    actual_cost_micro,
                    reference_id,
                    ..
                } => {
                    self.state.ledger.settle_reservation(
                        cycle_id,
                        &reserve_entry_id,
                        &reference_id,
                        actual_cost_micro,
                        Some(action_id),
                        self.state.version_tuple(),
                    )?;
                }
                SpineEvent::ActionRejected {
                    action_id,
                    reserve_entry_id,
                    reference_id,
                    ..
                } => {
                    self.state.ledger.refund_reservation(
                        cycle_id,
                        &reserve_entry_id,
                        &reference_id,
                        Some(action_id),
                        self.state.version_tuple(),
                    )?;
                }
                SpineEvent::ActionDeferred { .. } => {}
            }
        }

        Ok(())
    }

    fn apply_external_debits(&mut self, cycle_id: u64) -> Result<usize, NonCortexError> {
        let mut observations = self.external_debit_source.drain_observations();
        observations.sort_by(|lhs, rhs| lhs.reference_id.cmp(&rhs.reference_id));

        let mut applied = 0usize;
        for observation in observations {
            if self
                .state
                .seen_external_reference_ids
                .contains(&observation.reference_id)
            {
                continue;
            }

            if !self.matches_attribution(&observation) {
                continue;
            }

            self.state.ledger.apply_external_debit(
                cycle_id,
                &observation,
                self.state.version_tuple(),
            )?;

            self.state
                .seen_external_reference_ids
                .insert(observation.reference_id);
            applied = applied.saturating_add(1);
        }

        Ok(applied)
    }

    fn matches_attribution(&self, observation: &ExternalDebitObservation) -> bool {
        let Some(records) = self
            .state
            .attribution_index
            .get(&observation.cost_attribution_id)
        else {
            return false;
        };

        records.iter().any(|record| {
            let action_matches = observation
                .action_id
                .as_ref()
                .map(|action_id| action_id == &record.action_id)
                .unwrap_or(true);
            let cycle_matches = observation
                .cycle_id
                .map(|cycle_id| cycle_id == record.cycle_id)
                .unwrap_or(true);
            action_matches && cycle_matches
        })
    }
}
