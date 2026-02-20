use beluna::{
    cortex::{
        AttemptClampRequest, AttemptDraft, DeterministicAttemptClamp, ReactionLimits, derive_act_id,
    },
    types::{NeuralSignalDescriptor, NeuralSignalDescriptorCatalog, NeuralSignalType, is_uuid_v7},
};

fn catalog() -> NeuralSignalDescriptorCatalog {
    NeuralSignalDescriptorCatalog {
        version: "v1".to_string(),
        entries: vec![NeuralSignalDescriptor {
            r#type: NeuralSignalType::Act,
            endpoint_id: "ep.demo".to_string(),
            neural_signal_descriptor_id: "cap.demo".to_string(),
            payload_schema: serde_json::json!({"type":"object"}),
        }],
    }
}

fn draft() -> AttemptDraft {
    AttemptDraft {
        intent_span: "do thing".to_string(),
        based_on: vec!["sense:1".to_string()],
        attention_tags: vec!["a".to_string()],
        endpoint_id: "ep.demo".to_string(),
        neural_signal_descriptor_id: "cap.demo".to_string(),
        payload_draft: serde_json::json!({"ok": true}),
        goal_hint: None,
    }
}

#[test]
fn act_id_is_uuid_v7() {
    let lhs = derive_act_id(
        1,
        &["sense:1".to_string()],
        "ep.demo",
        "cap.demo",
        &serde_json::json!({"k":"v"}),
    );
    let rhs = derive_act_id(
        1,
        &["sense:1".to_string()],
        "ep.demo",
        "cap.demo",
        &serde_json::json!({"k":"v"}),
    );
    assert!(is_uuid_v7(&lhs));
    assert!(is_uuid_v7(&rhs));
    assert_ne!(lhs, rhs);
}

#[test]
fn clamp_rejects_unknown_sense_ids() {
    let clamp = DeterministicAttemptClamp;
    let result = clamp
        .clamp(AttemptClampRequest {
            cycle_id: 1,
            drafts: vec![draft()],
            neural_signal_descriptor_catalog: catalog(),
            known_sense_ids: vec!["sense:other".to_string()],
            limits: ReactionLimits::default(),
        })
        .expect("clamp should not hard-fail");

    assert!(result.acts.is_empty());
    assert_eq!(result.violations.len(), 1);
}

#[test]
fn clamp_emits_act_with_payload() {
    let clamp = DeterministicAttemptClamp;
    let valid = draft();
    let result = clamp
        .clamp(AttemptClampRequest {
            cycle_id: 1,
            drafts: vec![valid],
            neural_signal_descriptor_catalog: catalog(),
            known_sense_ids: vec!["sense:1".to_string()],
            limits: ReactionLimits::default(),
        })
        .expect("clamp should succeed");

    assert_eq!(result.acts.len(), 1);
    assert_eq!(result.acts[0].endpoint_id, "ep.demo");
    assert_eq!(result.acts[0].neural_signal_descriptor_id, "cap.demo");
    assert_eq!(result.acts[0].payload, serde_json::json!({"ok": true}));
}
