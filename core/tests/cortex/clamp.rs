use beluna::{
    cortex::{
        AttemptClampPort, AttemptClampRequest, AttemptDraft, CapabilityCatalog,
        DeterministicAttemptClamp, ReactionLimits, derive_act_id, is_uuid_v7,
    },
    runtime_types::RequestedResources,
};

fn catalog() -> CapabilityCatalog {
    CapabilityCatalog {
        version: "v1".to_string(),
        affordances: vec![beluna::cortex::AffordanceCapability {
            endpoint_id: "ep.demo".to_string(),
            allowed_capability_ids: vec!["cap.demo".to_string()],
            payload_schema: serde_json::json!({"type":"object"}),
            max_payload_bytes: 1024,
            default_resources: RequestedResources::default(),
        }],
    }
}

fn draft() -> AttemptDraft {
    AttemptDraft {
        intent_span: "do thing".to_string(),
        based_on: vec!["sense:1".to_string()],
        attention_tags: vec!["a".to_string()],
        endpoint_id: "ep.demo".to_string(),
        capability_id: "cap.demo".to_string(),
        capability_instance_id: "".to_string(),
        payload_draft: serde_json::json!({"ok": true}),
        requested_resources: RequestedResources {
            survival_micro: 42,
            time_ms: 1,
            io_units: 1,
            token_units: 0,
        },
        goal_hint: None,
    }
}

#[test]
fn act_id_is_uuid_v7() {
    let resources = RequestedResources {
        survival_micro: 12,
        time_ms: 3,
        io_units: 4,
        token_units: 5,
    };
    let lhs = derive_act_id(
        1,
        &["sense:1".to_string()],
        "ep.demo",
        "cap.demo",
        &serde_json::json!({"k":"v"}),
        &resources,
    );
    let rhs = derive_act_id(
        1,
        &["sense:1".to_string()],
        "ep.demo",
        "cap.demo",
        &serde_json::json!({"k":"v"}),
        &resources,
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
            capability_catalog: catalog(),
            known_sense_ids: vec!["sense:other".to_string()],
            limits: ReactionLimits::default(),
        })
        .expect("clamp should not hard-fail");

    assert!(result.acts.is_empty());
    assert_eq!(result.violations.len(), 1);
}

#[test]
fn clamp_emits_act_with_non_negative_survival() {
    let clamp = DeterministicAttemptClamp;
    let mut bad = draft();
    bad.requested_resources.survival_micro = -1;
    let result = clamp
        .clamp(AttemptClampRequest {
            cycle_id: 1,
            drafts: vec![bad],
            capability_catalog: catalog(),
            known_sense_ids: vec!["sense:1".to_string()],
            limits: ReactionLimits::default(),
        })
        .expect("clamp should succeed");

    assert_eq!(result.acts.len(), 1);
    assert_eq!(result.acts[0].requested_resources.survival_micro, 0);
}
