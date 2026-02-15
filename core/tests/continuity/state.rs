use beluna::{
    continuity::{ContinuityEngine, ContinuityState, DispatchContext},
    runtime_types::{
        Act, CapabilityDropPatch, CapabilityPatch, CognitionState, RequestedResources,
    },
    spine::types::{EndpointCapabilityDescriptor, RouteKey, SpineEvent},
};

fn descriptor(
    endpoint_id: &str,
    capability_id: &str,
    max_payload_bytes: usize,
) -> EndpointCapabilityDescriptor {
    EndpointCapabilityDescriptor {
        route: RouteKey {
            endpoint_id: endpoint_id.to_string(),
            capability_id: capability_id.to_string(),
        },
        payload_schema: serde_json::json!({"type":"object"}),
        max_payload_bytes,
        default_cost: beluna::spine::CostVector::default(),
        metadata: Default::default(),
    }
}

#[test]
fn arrival_order_wins_for_capability_patch_and_drop() {
    let mut engine = ContinuityEngine::new(ContinuityState::new());
    engine.apply_capability_patch(&CapabilityPatch {
        entries: vec![descriptor("ep.demo", "cap.demo", 256)],
    });
    engine.apply_capability_patch(&CapabilityPatch {
        entries: vec![descriptor("ep.demo", "cap.demo", 512)],
    });

    let snapshot = engine.capabilities_snapshot();
    let affordance = snapshot
        .resolve("ep.demo")
        .expect("ep.demo capability should exist");
    assert_eq!(affordance.max_payload_bytes, 512);

    engine.apply_capability_drop(&CapabilityDropPatch {
        routes: vec![RouteKey {
            endpoint_id: "ep.demo".to_string(),
            capability_id: "cap.demo".to_string(),
        }],
    });
    assert!(engine.capabilities_snapshot().resolve("ep.demo").is_none());

    engine.apply_capability_patch(&CapabilityPatch {
        entries: vec![descriptor("ep.demo", "cap.demo", 1024)],
    });
    let reintroduced_snapshot = engine.capabilities_snapshot();
    let reintroduced = reintroduced_snapshot
        .resolve("ep.demo")
        .expect("capability should be reintroduced after patch");
    assert_eq!(reintroduced.max_payload_bytes, 1024);
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
        act_id: "0194f1f3-cc2f-7aa7-8d4c-486f9f2f7c0a".to_string(),
        based_on: vec!["41f25f33-99f5-4250-99c3-020f8a92e199".to_string()],
        body_endpoint_name: "ep.demo".to_string(),
        capability_id: "cap.demo".to_string(),
        capability_instance_id: "instance:1".to_string(),
        normalized_payload: serde_json::json!({}),
        requested_resources: RequestedResources::default(),
    };

    engine
        .on_spine_event(
            &act,
            &SpineEvent::ActApplied {
                cycle_id: 1,
                seq_no: 1,
                act_id: act.act_id.clone(),
                capability_instance_id: act.capability_instance_id.clone(),
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
