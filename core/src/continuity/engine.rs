use std::sync::Arc;

use crate::{
    admission::{
        AdmissionResolver, AdmissionResolverConfig, AffordanceProfile, AffordanceRegistry,
        CostAdmissionPolicy, IntentAttempt, resolver::AdmissionContext,
    },
    continuity::{
        debit_sources::ExternalDebitSourcePort,
        error::{ContinuityError, invariant_violation},
        invariants::assert_settlement_consistency,
        ports::SpinePort,
        state::ContinuityState,
        types::{
            ContinuityCycleOutput, ExternalDebitObservation, NeuralSignalBatch, SenseSample,
            SituationView,
        },
    },
    cortex::SenseDelta,
    spine::{
        EndpointCapabilityDescriptor, EndpointExecutionOutcome, EndpointRegistration,
        EndpointRegistryPort, InMemoryEndpointRegistry, NativeFunctionEndpoint, RouteKey,
        RoutingSpineExecutor, SpineExecutionMode,
        types::{CostVector, SpineCapabilityCatalog, SpineEvent},
    },
};

pub struct ContinuityEngine {
    state: ContinuityState,
    admission: AdmissionResolver,
    spine: Arc<dyn SpinePort>,
    external_debit_source: Arc<dyn ExternalDebitSourcePort>,
}

impl ContinuityEngine {
    pub fn new(
        state: ContinuityState,
        admission: AdmissionResolver,
        spine: Arc<dyn SpinePort>,
        external_debit_source: Arc<dyn ExternalDebitSourcePort>,
    ) -> Self {
        Self {
            state,
            admission,
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

        let admission = AdmissionResolver::new(
            default_registry,
            CostAdmissionPolicy::default(),
            AdmissionResolverConfig::default(),
        );

        let registry: Arc<dyn EndpointRegistryPort> = Arc::new(InMemoryEndpointRegistry::new());
        let native_endpoint = Arc::new(NativeFunctionEndpoint::new(Arc::new(|invocation| {
            let action = invocation.action;
            Ok(EndpointExecutionOutcome::Applied {
                actual_cost_micro: action.reserved_cost.survival_micro,
                reference_id: format!("native:settle:{}", action.action_id),
            })
        })));

        let register = |affordance_key: &str, capability_handle: &str, default_cost: CostVector| {
            registry
                .register(
                    EndpointRegistration {
                        endpoint_id: format!("ep:native:{}:{}", affordance_key, capability_handle),
                        descriptor: EndpointCapabilityDescriptor {
                            route: RouteKey {
                                affordance_key: affordance_key.to_string(),
                                capability_handle: capability_handle.to_string(),
                            },
                            payload_schema: serde_json::json!({"type":"object"}),
                            max_payload_bytes: 16_384,
                            default_cost,
                            metadata: Default::default(),
                        },
                    },
                    native_endpoint.clone(),
                )
                .expect("default native endpoint registration should succeed");
        };

        register(
            "deliberate.plan",
            "cap.core",
            CostVector {
                survival_micro: 250,
                time_ms: 120,
                io_units: 1,
                token_units: 128,
            },
        );
        register(
            "deliberate.plan",
            "cap.core.lite",
            CostVector {
                survival_micro: 250,
                time_ms: 120,
                io_units: 1,
                token_units: 128,
            },
        );
        register(
            "deliberate.plan",
            "cap.core.minimal",
            CostVector {
                survival_micro: 250,
                time_ms: 120,
                io_units: 1,
                token_units: 128,
            },
        );
        register(
            "execute.tool",
            "cap.core",
            CostVector {
                survival_micro: 400,
                time_ms: 200,
                io_units: 2,
                token_units: 256,
            },
        );

        let spine_executor: Arc<dyn crate::spine::SpineExecutorPort> = Arc::new(
            RoutingSpineExecutor::new(SpineExecutionMode::SerializedDeterministic, registry),
        );

        Self::new(
            ContinuityState::new(initial_survival_micro),
            admission,
            Arc::new(crate::continuity::noop::SpinePortAdapter::new(
                spine_executor,
            )),
            Arc::new(crate::continuity::noop::NoopDebitSource),
        )
    }

    pub fn state(&self) -> &ContinuityState {
        &self.state
    }

    pub fn enqueue_sense_delta(&mut self, sense: SenseDelta, capacity: usize) -> bool {
        self.state.enqueue_sense_delta(sense, capacity)
    }

    pub fn sense_queue_len(&self) -> usize {
        self.state.sense_queue_len()
    }

    pub fn dequeue_sense_batch(&mut self, max_items: usize) -> Vec<SenseDelta> {
        self.state.dequeue_sense_batch(max_items)
    }

    pub fn enqueue_neural_signal_batch(
        &mut self,
        signal_batch: NeuralSignalBatch,
        capacity: usize,
    ) -> bool {
        self.state
            .enqueue_neural_signal_batch(signal_batch, capacity)
    }

    pub fn pop_neural_signal_batch(&mut self) -> Option<NeuralSignalBatch> {
        self.state.pop_neural_signal_batch()
    }

    pub fn ingest_sense(&mut self, sample: SenseSample) {
        self.state.ingest_sense(sample);
    }

    pub fn build_situation(&self) -> SituationView {
        self.state.build_situation()
    }

    pub fn capability_catalog_snapshot(&self) -> SpineCapabilityCatalog {
        self.spine.capability_catalog_snapshot()
    }

    pub async fn effectuate_attempts(
        &mut self,
        cycle_id: u64,
        attempts: Vec<IntentAttempt>,
    ) -> Result<ContinuityCycleOutput, ContinuityError> {
        self.state.cycle_id = cycle_id;

        let versions = self.state.version_tuple();
        let (admission_report, admitted_batch) = self.admission.admit_attempts(
            &mut AdmissionContext {
                ledger: &mut self.state.ledger,
                attribution_index: &mut self.state.attribution_index,
                policy_versions: versions,
            },
            cycle_id,
            attempts,
        )?;

        let admitted_action_count = admitted_batch.actions.len();
        let spine_report = self.spine.execute_admitted(admitted_batch).await?;

        self.reconcile_spine_report(cycle_id, &spine_report)?;

        let expired_reservations = self.state.ledger.expire_open_reservations(
            cycle_id,
            "expiry",
            self.state.version_tuple(),
        )?;

        let external_debit_applied_count = self.apply_external_debits(cycle_id)?;

        assert_settlement_consistency(&self.state)?;

        Ok(ContinuityCycleOutput {
            cycle_id,
            admission_report,
            spine_report,
            admitted_action_count,
            external_debit_applied_count,
            expired_reservation_count: expired_reservations.len(),
        })
    }

    pub async fn process_attempts(
        &mut self,
        cycle_id: u64,
        attempts: Vec<IntentAttempt>,
    ) -> Result<ContinuityCycleOutput, ContinuityError> {
        self.effectuate_attempts(cycle_id, attempts).await
    }

    fn reconcile_spine_report(
        &mut self,
        cycle_id: u64,
        spine_report: &crate::spine::types::SpineExecutionReport,
    ) -> Result<(), ContinuityError> {
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

    fn apply_external_debits(&mut self, cycle_id: u64) -> Result<usize, ContinuityError> {
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
