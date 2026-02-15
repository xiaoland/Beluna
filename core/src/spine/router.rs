use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    runtime_types::Act,
    spine::{
        error::SpineError,
        ports::{EndpointPort, EndpointRegistryPort, SpineExecutorPort},
        types::{EndpointExecutionOutcome, SpineCapabilityCatalog, SpineExecutionMode},
    },
};

type NativeEndpointHandler =
    dyn Fn(Act) -> Result<EndpointExecutionOutcome, SpineError> + Send + Sync;

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
    async fn invoke(&self, act: Act) -> Result<EndpointExecutionOutcome, SpineError> {
        (self.handler)(act)
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

    async fn invoke_one(&self, act: Act) -> EndpointExecutionOutcome {
        let Some(endpoint) = self.registry.resolve(&act.body_endpoint_name) else {
            return EndpointExecutionOutcome::Rejected {
                reason_code: "endpoint_not_found".to_string(),
                reference_id: format!("spine:missing_endpoint:{}", act.act_id),
            };
        };

        match endpoint.invoke(act.clone()).await {
            Ok(outcome) => outcome,
            Err(_) => EndpointExecutionOutcome::Rejected {
                reason_code: "endpoint_error".to_string(),
                reference_id: format!("spine:error:{}", act.act_id),
            },
        }
    }
}

#[async_trait]
impl SpineExecutorPort for RoutingSpineExecutor {
    fn mode(&self) -> SpineExecutionMode {
        self.mode
    }

    async fn dispatch_act(&self, act: Act) -> Result<EndpointExecutionOutcome, SpineError> {
        if act.act_id.trim().is_empty()
            || act.body_endpoint_name.trim().is_empty()
            || act.capability_id.trim().is_empty()
        {
            return Err(crate::spine::error::invalid_batch(
                "act dispatch is missing act_id/endpoint_id/capability_id",
            ));
        }

        Ok(self.invoke_one(act).await)
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
            EndpointCapabilityDescriptor, EndpointExecutionOutcome, InMemoryEndpointRegistry,
            RouteKey, RoutingSpineExecutor, SpineExecutionMode,
            error::{SpineError, backend_failure},
            ports::{EndpointPort, EndpointRegistryPort, SpineExecutorPort},
            types::CostVector,
        },
    };

    struct FailingEndpoint;

    #[async_trait]
    impl EndpointPort for FailingEndpoint {
        async fn invoke(&self, _act: Act) -> Result<EndpointExecutionOutcome, SpineError> {
            Err(backend_failure("endpoint exploded"))
        }
    }

    fn make_act(endpoint_id: &str, capability_id: &str) -> Act {
        Act {
            act_id: "act:1".to_string(),
            based_on: vec!["sense:1".to_string()],
            body_endpoint_name: endpoint_id.to_string(),
            capability_id: capability_id.to_string(),
            capability_instance_id: "instance:1".to_string(),
            normalized_payload: serde_json::json!({"ok":true}),
            requested_resources: RequestedResources::default(),
        }
    }

    #[tokio::test]
    async fn missing_endpoint_is_mapped_to_endpoint_not_found_rejection() {
        let registry: Arc<dyn EndpointRegistryPort> = Arc::new(InMemoryEndpointRegistry::new());
        let executor =
            RoutingSpineExecutor::new(SpineExecutionMode::SerializedDeterministic, registry);

        let outcome = executor
            .dispatch_act(make_act("core.mind", "missing.capability"))
            .await
            .expect("execution should succeed with per-act rejection");

        assert!(matches!(
            outcome,
            EndpointExecutionOutcome::Rejected { ref reason_code, .. } if reason_code == "endpoint_not_found"
        ));
    }

    #[tokio::test]
    async fn endpoint_error_is_mapped_to_endpoint_error_rejection() {
        let registry: Arc<dyn EndpointRegistryPort> = Arc::new(InMemoryEndpointRegistry::new());
        registry
            .register(
                EndpointCapabilityDescriptor {
                    route: RouteKey {
                        endpoint_id: "core.mind".to_string(),
                        capability_id: "observe.state".to_string(),
                    },
                    payload_schema: serde_json::json!({"type":"object"}),
                    max_payload_bytes: 1024,
                    default_cost: CostVector::default(),
                    metadata: Default::default(),
                },
                Arc::new(FailingEndpoint),
            )
            .expect("registration should succeed");

        let executor =
            RoutingSpineExecutor::new(SpineExecutionMode::SerializedDeterministic, registry);
        let outcome = executor
            .dispatch_act(make_act("core.mind", "observe.state"))
            .await
            .expect("execution should succeed with per-act rejection");

        assert!(matches!(
            outcome,
            EndpointExecutionOutcome::Rejected { ref reason_code, .. } if reason_code == "endpoint_error"
        ));
    }
}
