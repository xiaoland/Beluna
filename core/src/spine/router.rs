use std::sync::Arc;

use async_trait::async_trait;
use futures_util::future::join_all;

use crate::spine::{
    error::{SpineError, invalid_batch},
    ports::{EndpointPort, EndpointRegistryPort, SpineExecutorPort},
    types::{
        AdmittedAction, AdmittedActionBatch, EndpointExecutionOutcome, EndpointInvocation,
        OrderedSpineEvent, RouteKey, SpineCapabilityCatalog, SpineEvent, SpineExecutionMode,
        SpineExecutionReport,
    },
};

type NativeEndpointHandler =
    dyn Fn(EndpointInvocation) -> Result<EndpointExecutionOutcome, SpineError> + Send + Sync;

pub struct NativeFunctionEndpoint {
    handler: Arc<NativeEndpointHandler>,
}

impl NativeFunctionEndpoint {
    pub fn new(handler: Arc<NativeEndpointHandler>) -> Self {
        Self { handler }
    }
}

#[async_trait]
impl EndpointPort for NativeFunctionEndpoint {
    async fn invoke(
        &self,
        invocation: EndpointInvocation,
    ) -> Result<EndpointExecutionOutcome, SpineError> {
        (self.handler)(invocation)
    }
}

pub struct RoutingSpineExecutor {
    mode: SpineExecutionMode,
    registry: Arc<dyn EndpointRegistryPort>,
}

impl RoutingSpineExecutor {
    pub fn new(mode: SpineExecutionMode, registry: Arc<dyn EndpointRegistryPort>) -> Self {
        Self { mode, registry }
    }

    async fn execute_serialized(
        &self,
        admitted: AdmittedActionBatch,
    ) -> Result<SpineExecutionReport, SpineError> {
        let cycle_id = admitted.cycle_id;
        let mut events = Vec::with_capacity(admitted.actions.len());

        for (index, action) in admitted.actions.into_iter().enumerate() {
            let event = self.invoke_one(action).await;
            events.push(OrderedSpineEvent {
                seq_no: (index as u64) + 1,
                event,
            });
        }

        Ok(SpineExecutionReport {
            mode: self.mode,
            replay_cursor: Some(self.replay_cursor(cycle_id, events.len())),
            events,
        })
    }

    async fn execute_best_effort(
        &self,
        admitted: AdmittedActionBatch,
    ) -> Result<SpineExecutionReport, SpineError> {
        let cycle_id = admitted.cycle_id;

        let futures = admitted
            .actions
            .into_iter()
            .enumerate()
            .map(|(index, action)| async move {
                let event = self.invoke_one(action).await;
                OrderedSpineEvent {
                    seq_no: (index as u64) + 1,
                    event,
                }
            });

        let mut events = join_all(futures).await;
        events.sort_by_key(|item| item.seq_no);

        Ok(SpineExecutionReport {
            mode: self.mode,
            replay_cursor: Some(self.replay_cursor(cycle_id, events.len())),
            events,
        })
    }

    async fn invoke_one(&self, action: AdmittedAction) -> SpineEvent {
        let neural_signal_id = action.neural_signal_id.clone();
        let capability_instance_id = action.capability_instance_id.clone();
        let reserve_entry_id = action.reserve_entry_id.clone();
        let cost_attribution_id = action.cost_attribution_id.clone();

        let route = RouteKey {
            endpoint_id: action.endpoint_id.clone(),
            capability_id: action.capability_id.clone(),
        };

        let Some(endpoint) = self.registry.resolve(&route) else {
            return SpineEvent::ActionRejected {
                neural_signal_id,
                capability_instance_id,
                reserve_entry_id,
                cost_attribution_id,
                reason_code: "route_not_found".to_string(),
                reference_id: format!("spine:missing_route:{}", action.neural_signal_id),
            };
        };

        match endpoint.invoke(EndpointInvocation { action }).await {
            Ok(EndpointExecutionOutcome::Applied {
                actual_cost_micro,
                reference_id,
            }) => SpineEvent::ActionApplied {
                neural_signal_id,
                capability_instance_id,
                reserve_entry_id,
                cost_attribution_id,
                actual_cost_micro,
                reference_id,
            },
            Ok(EndpointExecutionOutcome::Rejected {
                reason_code,
                reference_id,
            }) => SpineEvent::ActionRejected {
                neural_signal_id,
                capability_instance_id,
                reserve_entry_id,
                cost_attribution_id,
                reason_code,
                reference_id,
            },
            Ok(EndpointExecutionOutcome::Deferred { reason_code }) => SpineEvent::ActionDeferred {
                neural_signal_id,
                capability_instance_id,
                reason_code,
            },
            Err(_) => {
                let reference_id = format!("spine:error:{}", cost_attribution_id);
                SpineEvent::ActionRejected {
                    neural_signal_id,
                    capability_instance_id,
                    reserve_entry_id,
                    cost_attribution_id,
                    reason_code: "endpoint_error".to_string(),
                    reference_id,
                }
            }
        }
    }

    fn replay_cursor(&self, cycle_id: u64, event_count: usize) -> String {
        let version = self.registry.catalog_snapshot().version;
        format!("route:{}:{}:{}", cycle_id, event_count, version)
    }
}

#[async_trait]
impl SpineExecutorPort for RoutingSpineExecutor {
    fn mode(&self) -> SpineExecutionMode {
        self.mode
    }

    async fn execute_admitted(
        &self,
        admitted: AdmittedActionBatch,
    ) -> Result<SpineExecutionReport, SpineError> {
        if admitted
            .actions
            .iter()
            .any(|action| action.neural_signal_id.is_empty() || action.reserve_entry_id.is_empty())
        {
            return Err(invalid_batch(
                "admitted action is missing neural_signal_id or reserve_entry_id",
            ));
        }

        match self.mode {
            SpineExecutionMode::SerializedDeterministic => self.execute_serialized(admitted).await,
            SpineExecutionMode::BestEffortReplayable => self.execute_best_effort(admitted).await,
        }
    }

    fn capability_catalog_snapshot(&self) -> SpineCapabilityCatalog {
        self.registry.catalog_snapshot()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use crate::spine::{
        EndpointCapabilityDescriptor, EndpointExecutionOutcome, EndpointInvocation,
        EndpointRegistration, InMemoryEndpointRegistry, RouteKey, RoutingSpineExecutor,
        SpineExecutionMode,
        error::{SpineError, backend_failure},
        ports::{EndpointPort, EndpointRegistryPort, SpineExecutorPort},
        types::{AdmittedAction, AdmittedActionBatch, CostVector, SpineEvent},
    };

    struct FailingEndpoint;

    #[async_trait]
    impl EndpointPort for FailingEndpoint {
        async fn invoke(
            &self,
            _invocation: EndpointInvocation,
        ) -> Result<EndpointExecutionOutcome, SpineError> {
            Err(backend_failure("endpoint exploded"))
        }
    }

    fn make_action(endpoint_id: &str, capability_id: &str) -> AdmittedAction {
        AdmittedAction {
            neural_signal_id: "ns:1".to_string(),
            capability_instance_id: "plan.instance".to_string(),
            source_attempt_id: "att:1".to_string(),
            reserve_entry_id: "res:1".to_string(),
            cost_attribution_id: "cost:1".to_string(),
            endpoint_id: endpoint_id.to_string(),
            capability_id: capability_id.to_string(),
            normalized_payload: serde_json::json!({"ok":true}),
            reserved_cost: CostVector::default(),
            degraded: false,
            degradation_profile_id: None,
            admission_cycle: 1,
            metadata: Default::default(),
        }
    }

    #[tokio::test]
    async fn missing_route_is_mapped_to_route_not_found_rejection() {
        let registry: Arc<dyn EndpointRegistryPort> = Arc::new(InMemoryEndpointRegistry::new());
        let executor =
            RoutingSpineExecutor::new(SpineExecutionMode::SerializedDeterministic, registry);

        let report = executor
            .execute_admitted(AdmittedActionBatch {
                cycle_id: 1,
                actions: vec![make_action("core.mind", "missing.capability")],
            })
            .await
            .expect("execution should succeed with per-action rejection");

        assert_eq!(report.events.len(), 1);
        assert!(matches!(
            &report.events[0].event,
            SpineEvent::ActionRejected { reason_code, .. } if reason_code == "route_not_found"
        ));
    }

    #[tokio::test]
    async fn endpoint_error_is_mapped_to_endpoint_error_rejection() {
        let registry: Arc<dyn EndpointRegistryPort> = Arc::new(InMemoryEndpointRegistry::new());
        registry
            .register(
                EndpointRegistration {
                    endpoint_id: "ep:observe:core".to_string(),
                    descriptor: EndpointCapabilityDescriptor {
                        route: RouteKey {
                            endpoint_id: "core.mind".to_string(),
                            capability_id: "observe.state".to_string(),
                        },
                        payload_schema: serde_json::json!({"type":"object"}),
                        max_payload_bytes: 1024,
                        default_cost: CostVector::default(),
                        metadata: Default::default(),
                    },
                },
                Arc::new(FailingEndpoint),
            )
            .expect("registration should succeed");

        let executor =
            RoutingSpineExecutor::new(SpineExecutionMode::SerializedDeterministic, registry);
        let report = executor
            .execute_admitted(AdmittedActionBatch {
                cycle_id: 1,
                actions: vec![make_action("core.mind", "observe.state")],
            })
            .await
            .expect("execution should succeed with per-action rejection");

        assert_eq!(report.events.len(), 1);
        assert!(matches!(
            &report.events[0].event,
            SpineEvent::ActionRejected { reason_code, .. } if reason_code == "endpoint_error"
        ));
    }
}
