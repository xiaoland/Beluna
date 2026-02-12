use beluna::cortex::{
    AttemptClampPort, AttemptClampRequest, AttemptDraft, CapabilityCatalog, DeterministicAttemptClamp,
    ReactionLimits, SenseDelta, derive_attempt_id,
};

fn catalog() -> CapabilityCatalog {
    CapabilityCatalog {
        version: "v1".to_string(),
        affordances: vec![beluna::cortex::AffordanceCapability {
            affordance_key: "deliberate.plan".to_string(),
            allowed_capability_handles: vec!["cap.core".to_string()],
            payload_schema: serde_json::json!({"type":"object","required":["ok"],"properties":{"ok":{"type":"boolean"}}}),
            max_payload_bytes: 2048,
            default_resources: Default::default(),
        }],
    }
}

fn catalog_with_empty_capability_list() -> CapabilityCatalog {
    CapabilityCatalog {
        version: "v1".to_string(),
        affordances: vec![beluna::cortex::AffordanceCapability {
            affordance_key: "deliberate.plan".to_string(),
            allowed_capability_handles: vec![],
            payload_schema: serde_json::json!({"type":"object"}),
            max_payload_bytes: 2048,
            default_resources: Default::default(),
        }],
    }
}

fn catalog_with_invalid_schema() -> CapabilityCatalog {
    CapabilityCatalog {
        version: "v1".to_string(),
        affordances: vec![beluna::cortex::AffordanceCapability {
            affordance_key: "deliberate.plan".to_string(),
            allowed_capability_handles: vec!["cap.core".to_string()],
            payload_schema: serde_json::json!({"type":"does-not-exist"}),
            max_payload_bytes: 2048,
            default_resources: Default::default(),
        }],
    }
}

fn sense_window() -> Vec<SenseDelta> {
    vec![SenseDelta {
        sense_id: "s1".to_string(),
        source: "sensor".to_string(),
        payload: serde_json::json!({"v": 1}),
    }]
}

fn valid_draft() -> AttemptDraft {
    AttemptDraft {
        intent_span: "analyze and plan".to_string(),
        based_on: vec!["s1".to_string()],
        attention_tags: vec!["plan".to_string()],
        affordance_key: "deliberate.plan".to_string(),
        capability_handle: "cap.core".to_string(),
        payload_draft: serde_json::json!({"ok": true}),
        requested_resources: Default::default(),
        commitment_hint: Some("c1".to_string()),
        goal_hint: Some("g1".to_string()),
    }
}

#[test]
fn given_valid_draft_when_clamped_then_attempt_contains_attempt_id_and_based_on() {
    let clamp = DeterministicAttemptClamp;
    let result = clamp
        .clamp(AttemptClampRequest {
            reaction_id: 1,
            drafts: vec![valid_draft()],
            capability_catalog: catalog(),
            sense_window: sense_window(),
            limits: ReactionLimits::default(),
        })
        .expect("clamp should succeed");

    assert_eq!(result.attempts.len(), 1);
    assert!(!result.attempts[0].attempt_id.is_empty());
    assert_eq!(result.attempts[0].based_on, vec!["s1".to_string()]);
}

#[test]
fn given_unknown_affordance_when_clamped_then_attempt_is_dropped() {
    let clamp = DeterministicAttemptClamp;
    let mut draft = valid_draft();
    draft.affordance_key = "unknown.affordance".to_string();

    let result = clamp
        .clamp(AttemptClampRequest {
            reaction_id: 2,
            drafts: vec![draft],
            capability_catalog: catalog(),
            sense_window: sense_window(),
            limits: ReactionLimits::default(),
        })
        .expect("clamp should succeed");

    assert!(result.attempts.is_empty());
}

#[test]
fn given_schema_violation_when_clamped_then_attempt_is_dropped() {
    let clamp = DeterministicAttemptClamp;
    let mut draft = valid_draft();
    draft.payload_draft = serde_json::json!({"ok":"not-bool"});

    let result = clamp
        .clamp(AttemptClampRequest {
            reaction_id: 3,
            drafts: vec![draft],
            capability_catalog: catalog(),
            sense_window: sense_window(),
            limits: ReactionLimits::default(),
        })
        .expect("clamp should succeed");

    assert!(result.attempts.is_empty());
}

#[test]
fn given_empty_capability_list_when_clamped_then_attempt_is_dropped() {
    let clamp = DeterministicAttemptClamp;
    let result = clamp
        .clamp(AttemptClampRequest {
            reaction_id: 30,
            drafts: vec![valid_draft()],
            capability_catalog: catalog_with_empty_capability_list(),
            sense_window: sense_window(),
            limits: ReactionLimits::default(),
        })
        .expect("clamp should succeed");

    assert!(result.attempts.is_empty());
}

#[test]
fn given_invalid_catalog_schema_when_clamped_then_attempt_is_dropped_without_cycle_error() {
    let clamp = DeterministicAttemptClamp;
    let result = clamp
        .clamp(AttemptClampRequest {
            reaction_id: 31,
            drafts: vec![valid_draft()],
            capability_catalog: catalog_with_invalid_schema(),
            sense_window: sense_window(),
            limits: ReactionLimits::default(),
        })
        .expect("clamp should succeed and drop draft deterministically");

    assert!(result.attempts.is_empty());
    assert!(!result.violations.is_empty());
}

#[test]
fn given_max_attempts_when_clamped_then_output_is_stable_and_truncated() {
    let clamp = DeterministicAttemptClamp;
    let mut limits = ReactionLimits::default();
    limits.max_attempts = 2;

    let mut d1 = valid_draft();
    d1.payload_draft = serde_json::json!({"ok":true,"id":1});
    let mut d2 = valid_draft();
    d2.payload_draft = serde_json::json!({"ok":true,"id":2});
    let mut d3 = valid_draft();
    d3.payload_draft = serde_json::json!({"ok":true,"id":3});

    let lhs = clamp
        .clamp(AttemptClampRequest {
            reaction_id: 4,
            drafts: vec![d1.clone(), d2.clone(), d3.clone()],
            capability_catalog: catalog(),
            sense_window: sense_window(),
            limits: limits.clone(),
        })
        .expect("clamp should succeed");
    let rhs = clamp
        .clamp(AttemptClampRequest {
            reaction_id: 4,
            drafts: vec![d1, d2, d3],
            capability_catalog: catalog(),
            sense_window: sense_window(),
            limits,
        })
        .expect("clamp should succeed");

    assert_eq!(lhs.attempts.len(), 2);
    assert_eq!(lhs.attempts, rhs.attempts);
}

#[test]
fn given_equivalent_inputs_when_deriving_attempt_id_then_output_is_stable() {
    let payload_a = serde_json::json!({"b": 2, "a": 1});
    let payload_b = serde_json::json!({"a": 1, "b": 2});
    let resources = beluna::admission::RequestedResources {
        survival_micro: 1,
        time_ms: 2,
        io_units: 3,
        token_units: 4,
    };

    let lhs = derive_attempt_id(
        10,
        &["s1".to_string()],
        "deliberate.plan",
        "cap.core",
        &payload_a,
        &resources,
        "cat:123",
    );
    let rhs = derive_attempt_id(
        10,
        &["s1".to_string()],
        "deliberate.plan",
        "cap.core",
        &payload_b,
        &resources,
        "cat:123",
    );

    assert_eq!(lhs, rhs);
}
