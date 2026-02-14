use std::sync::Arc;

use async_trait::async_trait;

use crate::spine::{
    error::SpineError,
    ports::{EndpointPort, EndpointRegistryPort, SpineExecutorPort},
    types::{
        ActDispatchRequest, EndpointExecutionOutcome, EndpointInvocation, SpineCapabilityCatalog,
        SpineEvent, SpineExecutionMode,
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

    async fn invoke_one(&self, request: ActDispatchRequest) -> SpineEvent {
        let route = crate::spine::types::RouteKey {
            endpoint_id: request.act.endpoint_id.clone(),
            capability_id: request.act.capability_id.clone(),
        };

        let Some(endpoint) = self.registry.resolve(&route) else {
            return SpineEvent::ActRejected {
                cycle_id: request.cycle_id,
                seq_no: request.seq_no,
                act_id: request.act.act_id.clone(),
                capability_instance_id: request.act.capability_instance_id.clone(),
                reserve_entry_id: request.reserve_entry_id.clone(),
                cost_attribution_id: request.cost_attribution_id.clone(),
                reason_code: "route_not_found".to_string(),
                reference_id: format!("spine:missing_route:{}", request.act.act_id),
            };
        };

        match endpoint
            .invoke(EndpointInvocation {
                request: request.clone(),
            })
            .await
        {
            Ok(EndpointExecutionOutcome::Applied {
                actual_cost_micro,
                reference_id,
            }) => SpineEvent::ActApplied {
                cycle_id: request.cycle_id,
                seq_no: request.seq_no,
                act_id: request.act.act_id,
                capability_instance_id: request.act.capability_instance_id,
                reserve_entry_id: request.reserve_entry_id,
                cost_attribution_id: request.cost_attribution_id,
                actual_cost_micro,
                reference_id,
            },
            Ok(EndpointExecutionOutcome::Rejected {
                reason_code,
                reference_id,
            }) => SpineEvent::ActRejected {
                cycle_id: request.cycle_id,
                seq_no: request.seq_no,
                act_id: request.act.act_id,
                capability_instance_id: request.act.capability_instance_id,
                reserve_entry_id: request.reserve_entry_id,
                cost_attribution_id: request.cost_attribution_id,
                reason_code,
                reference_id,
            },
            Ok(EndpointExecutionOutcome::Deferred {
                reason_code,
                reference_id,
            }) => SpineEvent::ActDeferred {
                cycle_id: request.cycle_id,
                seq_no: request.seq_no,
                act_id: request.act.act_id,
                capability_instance_id: request.act.capability_instance_id,
                reserve_entry_id: request.reserve_entry_id,
                cost_attribution_id: request.cost_attribution_id,
                reason_code,
                reference_id,
            },
            Err(_) => SpineEvent::ActRejected {
                cycle_id: request.cycle_id,
                seq_no: request.seq_no,
                act_id: request.act.act_id,
                capability_instance_id: request.act.capability_instance_id,
                reserve_entry_id: request.reserve_entry_id,
                cost_attribution_id: request.cost_attribution_id.clone(),
                reason_code: "endpoint_error".to_string(),
                reference_id: format!("spine:error:{}", request.cost_attribution_id),
            },
        }
    }
}

#[async_trait]
impl SpineExecutorPort for RoutingSpineExecutor {
    fn mode(&self) -> SpineExecutionMode {
        self.mode
    }

    async fn dispatch_act(&self, request: ActDispatchRequest) -> Result<SpineEvent, SpineError> {
        if request.act.act_id.trim().is_empty()
            || request.reserve_entry_id.trim().is_empty()
            || request.cost_attribution_id.trim().is_empty()
        {
            return Err(crate::spine::error::invalid_batch(
                "act dispatch request is missing act_id/reserve_entry_id/cost_attribution_id",
            ));
        }

        Ok(self.invoke_one(request).await)
    }

    fn capability_catalog_snapshot(&self) -> SpineCapabilityCatalog {
        self.registry.catalog_snapshot()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use crate::{
        runtime_types::{Act, RequestedResources},
        spine::{
            EndpointCapabilityDescriptor, EndpointExecutionOutcome, EndpointInvocation,
            EndpointRegistration, InMemoryEndpointRegistry, RouteKey, RoutingSpineExecutor,
            SpineExecutionMode,
            error::{SpineError, backend_failure},
            ports::{EndpointPort, EndpointRegistryPort, SpineExecutorPort},
            types::{ActDispatchRequest, CostVector, SpineEvent},
        },
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

    fn make_request(endpoint_id: &str, capability_id: &str) -> ActDispatchRequest {
        ActDispatchRequest {
            cycle_id: 1,
            seq_no: 1,
            act: Act {
                act_id: "act:1".to_string(),
                based_on: vec!["sense:1".to_string()],
                endpoint_id: endpoint_id.to_string(),
                capability_id: capability_id.to_string(),
                capability_instance_id: "instance:1".to_string(),
                normalized_payload: serde_json::json!({"ok":true}),
                requested_resources: RequestedResources::default(),
            },
            reserve_entry_id: "res:1".to_string(),
            cost_attribution_id: "cost:1".to_string(),
        }
    }

    #[tokio::test]
    async fn missing_route_is_mapped_to_route_not_found_rejection() {
        let registry: Arc<dyn EndpointRegistryPort> = Arc::new(InMemoryEndpointRegistry::new());
        let executor =
            RoutingSpineExecutor::new(SpineExecutionMode::SerializedDeterministic, registry);

        let event = executor
            .dispatch_act(make_request("core.mind", "missing.capability"))
            .await
            .expect("execution should succeed with per-act rejection");

        assert!(matches!(
            event,
            SpineEvent::ActRejected { ref reason_code, .. } if reason_code == "route_not_found"
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
        let event = executor
            .dispatch_act(make_request("core.mind", "observe.state"))
            .await
            .expect("execution should succeed with per-act rejection");

        assert!(matches!(
            event,
            SpineEvent::ActRejected { ref reason_code, .. } if reason_code == "endpoint_error"
        ));
    }
}
