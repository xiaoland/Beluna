use std::sync::Arc;

use beluna::{
    runtime_types::{Act, RequestedResources},
    spine::{
        DeterministicNoopSpine, InMemoryEndpointRegistry, RoutingSpineExecutor, SpineExecutionMode,
        SpineExecutorPort,
        types::{CostVector, EndpointExecutionOutcome},
    },
};

fn request(body_endpoint_name: &str, capability_id: &str, seq_no: u64) -> Act {
    Act {
        act_id: uuid::Uuid::now_v7().to_string(),
        based_on: vec!["41f25f33-99f5-4250-99c3-020f8a92e199".to_string()],
        body_endpoint_name: body_endpoint_name.to_string(),
        capability_id: capability_id.to_string(),
        capability_instance_id: format!("instance:{seq_no}"),
        normalized_payload: serde_json::json!({"ok":true}),
        requested_resources: RequestedResources {
            survival_micro: 10,
            time_ms: 1,
            io_units: 1,
            token_units: 0,
        },
    }
}

#[tokio::test]
async fn noop_spine_dispatches_single_act() {
    let spine = DeterministicNoopSpine::new(SpineExecutionMode::SerializedDeterministic);
    let outcome = spine
        .dispatch_act(request("ep.demo", "cap.demo", 1))
        .await
        .expect("noop dispatch should succeed");

    assert!(matches!(
        outcome,
        EndpointExecutionOutcome::Applied {
            actual_cost_micro: 10,
            ..
        }
    ));
}

#[tokio::test]
async fn routing_spine_rejects_missing_endpoint() {
    let registry = Arc::new(InMemoryEndpointRegistry::new());
    let spine = RoutingSpineExecutor::new(SpineExecutionMode::SerializedDeterministic, registry);

    let outcome = spine
        .dispatch_act(request("missing.endpoint", "missing.capability", 3))
        .await
        .expect("routing should produce outcome");

    assert!(matches!(
        outcome,
        EndpointExecutionOutcome::Rejected { ref reason_code, .. } if reason_code == "endpoint_not_found"
    ));
}

#[test]
fn cost_vector_is_default_zeroed() {
    assert_eq!(CostVector::default().survival_micro, 0);
}
