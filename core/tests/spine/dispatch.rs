use std::sync::Arc;

use beluna::{
    runtime_types::{Act, RequestedResources},
    spine::{
        DeterministicNoopSpine, InMemoryEndpointRegistry, RoutingSpineExecutor, SpineEvent,
        SpineExecutionMode, SpineExecutorPort,
        types::{ActDispatchRequest, CostVector},
    },
};

fn request(endpoint_id: &str, capability_id: &str, seq_no: u64) -> ActDispatchRequest {
    ActDispatchRequest {
        cycle_id: 1,
        seq_no,
        act: Act {
            act_id: format!("act:{seq_no}"),
            based_on: vec!["sense:1".to_string()],
            endpoint_id: endpoint_id.to_string(),
            capability_id: capability_id.to_string(),
            capability_instance_id: format!("instance:{seq_no}"),
            normalized_payload: serde_json::json!({"ok":true}),
            requested_resources: RequestedResources {
                survival_micro: 10,
                time_ms: 1,
                io_units: 1,
                token_units: 0,
            },
        },
        reserve_entry_id: format!("res:{seq_no}"),
        cost_attribution_id: format!("cat:{seq_no}"),
    }
}

#[tokio::test]
async fn noop_spine_dispatches_single_act() {
    let spine = DeterministicNoopSpine::new(SpineExecutionMode::SerializedDeterministic);
    let event = spine
        .dispatch_act(request("ep.demo", "cap.demo", 1))
        .await
        .expect("noop dispatch should succeed");

    assert!(matches!(
        event,
        SpineEvent::ActApplied {
            seq_no: 1,
            actual_cost_micro: 10,
            ..
        }
    ));
}

#[tokio::test]
async fn routing_spine_rejects_missing_route() {
    let registry = Arc::new(InMemoryEndpointRegistry::new());
    let spine = RoutingSpineExecutor::new(SpineExecutionMode::SerializedDeterministic, registry);

    let event = spine
        .dispatch_act(request("missing.endpoint", "missing.capability", 3))
        .await
        .expect("routing should produce event");

    assert!(matches!(
        event,
        SpineEvent::ActRejected {
            seq_no: 3,
            ref reason_code,
            ..
        } if reason_code == "route_not_found"
    ));
}

#[test]
fn cost_vector_is_default_zeroed() {
    assert_eq!(CostVector::default().survival_micro, 0);
}
