use std::sync::Arc;

use beluna::{
    afferent_pathway::SenseAfferentPathway,
    config::SpineRuntimeConfig,
    spine::{ActDispatchResult, EndpointBinding, NativeFunctionEndpoint, Spine},
    types::{Act, NeuralSignalDescriptor, NeuralSignalType},
};

fn request(endpoint_id: &str, neural_signal_descriptor_id: &str, seq_no: u64) -> Act {
    Act {
        act_id: format!("act:{seq_no}"),
        endpoint_id: endpoint_id.to_string(),
        neural_signal_descriptor_id: neural_signal_descriptor_id.to_string(),
        payload: serde_json::json!({"ok": true}),
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
            vec![NeuralSignalDescriptor {
                r#type: NeuralSignalType::Act,
                endpoint_id: "placeholder".to_string(),
                neural_signal_descriptor_id: "cap.demo".to_string(),
                payload_schema: serde_json::json!({"type":"object"}),
            }],
        )
        .expect("endpoint registration should succeed");

    let outcome = spine
        .dispatch_act(request(&handle.body_endpoint_id, "cap.demo", 1))
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
fn catalog_snapshot_contains_registered_descriptor() {
    let spine = test_spine();
    let endpoint = Arc::new(NativeFunctionEndpoint::new(Arc::new(|act| {
        Ok(ActDispatchResult::Acknowledged {
            reference_id: format!("native:settle:{}", act.act_id),
        })
    })));
    let handle = spine
        .add_endpoint(
            "ep.catalog",
            EndpointBinding::Inline(endpoint),
            vec![NeuralSignalDescriptor {
                r#type: NeuralSignalType::Act,
                endpoint_id: "placeholder".to_string(),
                neural_signal_descriptor_id: "cap.catalog".to_string(),
                payload_schema: serde_json::json!({"type":"object"}),
            }],
        )
        .expect("endpoint registration should succeed");

    let catalog = spine.neural_signal_descriptor_catalog_snapshot();
    assert!(catalog.entries.iter().any(|entry| {
        entry.r#type == NeuralSignalType::Act
            && entry.endpoint_id == handle.body_endpoint_id
            && entry.neural_signal_descriptor_id == "cap.catalog"
    }));
}
