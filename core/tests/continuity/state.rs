use beluna::{
    continuity::{ContinuityEngine, ContinuityState, DispatchContext},
    spine::types::SpineEvent,
    types::{
        Act, CognitionState, NeuralSignalDescriptor, NeuralSignalDescriptorDropPatch,
        NeuralSignalDescriptorPatch, NeuralSignalDescriptorRouteKey, NeuralSignalType,
    },
};

fn descriptor(
    endpoint_id: &str,
    neural_signal_descriptor_id: &str,
    schema_revision: u64,
) -> NeuralSignalDescriptor {
    NeuralSignalDescriptor {
        r#type: NeuralSignalType::Act,
        endpoint_id: endpoint_id.to_string(),
        neural_signal_descriptor_id: neural_signal_descriptor_id.to_string(),
        payload_schema: serde_json::json!({
            "type": "object",
            "schema_revision": schema_revision
        }),
    }
}

#[test]
fn arrival_order_wins_for_capability_patch_and_drop() {
    let mut engine = ContinuityEngine::new(ContinuityState::new());
    engine.apply_neural_signal_descriptor_patch(&NeuralSignalDescriptorPatch {
        entries: vec![descriptor("ep.demo", "cap.demo", 1)],
    });
    engine.apply_neural_signal_descriptor_patch(&NeuralSignalDescriptorPatch {
        entries: vec![descriptor("ep.demo", "cap.demo", 2)],
    });

    let snapshot = engine.neural_signal_descriptor_snapshot();
    let latest_descriptor = snapshot
        .entries
        .iter()
        .find(|entry| {
            entry.r#type == NeuralSignalType::Act
                && entry.endpoint_id == "ep.demo"
                && entry.neural_signal_descriptor_id == "cap.demo"
        })
        .expect("ep.demo/cap.demo descriptor should exist");
    assert_eq!(
        latest_descriptor.payload_schema["schema_revision"],
        serde_json::json!(2)
    );

    engine.apply_neural_signal_descriptor_drop(&NeuralSignalDescriptorDropPatch {
        routes: vec![NeuralSignalDescriptorRouteKey {
            r#type: NeuralSignalType::Act,
            endpoint_id: "ep.demo".to_string(),
            neural_signal_descriptor_id: "cap.demo".to_string(),
        }],
    });
    assert!(
        engine
            .neural_signal_descriptor_snapshot()
            .entries
            .iter()
            .all(|entry| {
                !(entry.r#type == NeuralSignalType::Act
                    && entry.endpoint_id == "ep.demo"
                    && entry.neural_signal_descriptor_id == "cap.demo")
            })
    );

    engine.apply_neural_signal_descriptor_patch(&NeuralSignalDescriptorPatch {
        entries: vec![descriptor("ep.demo", "cap.demo", 3)],
    });
    let reintroduced_snapshot = engine.neural_signal_descriptor_snapshot();
    let reintroduced = reintroduced_snapshot
        .entries
        .iter()
        .find(|entry| {
            entry.r#type == NeuralSignalType::Act
                && entry.endpoint_id == "ep.demo"
                && entry.neural_signal_descriptor_id == "cap.demo"
        })
        .expect("capability should be reintroduced after patch");
    assert_eq!(
        reintroduced.payload_schema["schema_revision"],
        serde_json::json!(3)
    );
}

#[test]
fn cognition_state_is_persisted() {
    let mut engine = ContinuityEngine::with_defaults();
    assert_eq!(engine.cognition_state_snapshot().revision, 0);

    engine
        .persist_cognition_state(CognitionState {
            revision: 4,
            goal_stack: vec![],
        })
        .expect("persist should succeed");

    assert_eq!(engine.cognition_state_snapshot().revision, 4);
}

#[test]
fn spine_events_are_recorded() {
    let mut engine = ContinuityEngine::with_defaults();
    let act = Act {
        act_id: "act:1".to_string(),
        endpoint_id: "ep.demo".to_string(),
        neural_signal_descriptor_id: "cap.demo".to_string(),
        payload: serde_json::json!({}),
    };

    engine
        .on_spine_event(
            &act,
            &SpineEvent::ActApplied {
                cycle_id: 1,
                seq_no: 1,
                act_id: act.act_id.clone(),
                reserve_entry_id: "res:1".to_string(),
                cost_attribution_id: "cat:1".to_string(),
                actual_cost_micro: 1,
                reference_id: "ref:1".to_string(),
            },
            &DispatchContext {
                cycle_id: 1,
                act_seq_no: 1,
            },
        )
        .expect("event ingestion should succeed");

    let records = engine.dispatch_records();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].event, "act_applied");
}
