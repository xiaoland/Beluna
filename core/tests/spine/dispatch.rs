use std::sync::Arc;

use beluna::{
    afferent_pathway::SenseAfferentPathway,
    config::SpineRuntimeConfig,
    spine::{
        ActDispatchResult, CostVector, EndpointBinding, EndpointCapabilityDescriptor,
        NativeFunctionEndpoint, RouteKey, Spine,
    },
    types::{Act, RequestedResources},
};

fn request(body_endpoint_name: &str, capability_id: &str, seq_no: u64) -> Act {
    Act {
        act_id: format!("act:{seq_no}"),
        based_on: vec!["sense:1".to_string()],
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

fn test_spine() -> Arc<Spine> {
    let config = SpineRuntimeConfig { adapters: vec![] };
    Spine::new(&config, SenseAfferentPathway::new(4).0)
}

#[tokio::test]
async fn noop_equivalent_spine_dispatches_single_act() {
    let spine = test_spine();
    let endpoint = Arc::new(NativeFunctionEndpoint::new(Arc::new(|act| {
        Ok(ActDispatchResult::Acknowledged {
            reference_id: format!("native:settle:{}", act.act_id),
        })
    })));
    let handle = spine
        .add_endpoint(
            "ep.demo",
            EndpointBinding::Inline(endpoint),
            vec![EndpointCapabilityDescriptor {
                route: RouteKey {
                    endpoint_id: "placeholder".to_string(),
                    capability_id: "cap.demo".to_string(),
                },
                payload_schema: serde_json::json!({"type":"object"}),
                max_payload_bytes: 1024,
                default_cost: CostVector::default(),
                metadata: Default::default(),
            }],
        )
        .expect("endpoint registration should succeed");

    let outcome = spine
        .dispatch_act(request(&handle.body_endpoint_name, "cap.demo", 1))
        .await
        .expect("dispatch should succeed");

    assert!(matches!(outcome, ActDispatchResult::Acknowledged { .. }));
}

#[tokio::test]
async fn spine_rejects_missing_endpoint() {
    let spine = test_spine();

    let outcome = spine
        .dispatch_act(request("missing.endpoint", "missing.capability", 3))
        .await
        .expect("dispatch should produce outcome");

    assert!(matches!(
        outcome,
        ActDispatchResult::Rejected { ref reason_code, .. } if reason_code == "endpoint_not_found"
    ));
}

#[test]
fn cost_vector_is_default_zeroed() {
    assert_eq!(CostVector::default().survival_micro, 0);
}
