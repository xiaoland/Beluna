use serde_json::json;

use super::flatten::{EventLevel, flatten_contract_event};
use crate::observability::contract::{
    ContractEvent, CortexOrganExecutionEvent, DispatchOutcomeClass, OrganResponseStatus,
    StemAfferentEvent, StemEfferentEvent,
};

#[test]
fn flattens_cortex_primary_error_as_warning() {
    let flat = flatten_contract_event(&ContractEvent::CortexPrimary(CortexOrganExecutionEvent {
        run_id: "run.1".to_string(),
        timestamp: "2026-03-28T10:00:00Z".to_string(),
        tick: 42,
        request_id: "req-1".to_string(),
        span_id: "req-1".to_string(),
        parent_span_id: None,
        phase: "end".to_string(),
        status: OrganResponseStatus::Error,
        route_or_backend: Some("primary-route".to_string()),
        input_payload: None,
        output_payload: None,
        error: Some(json!({"code": "timeout"})),
        ai_request_id: None,
        thread_id: None,
        turn_id: None,
    }));

    assert_eq!(flat.level, EventLevel::Warn);
    assert_eq!(flat.family, "cortex.primary");
    assert_eq!(flat.tick, 42);
    assert_eq!(flat.request_id.as_deref(), Some("req-1"));
    assert_eq!(flat.outcome.as_deref(), Some("error"));
    assert_eq!(flat.state.as_deref(), Some("end"));
}

#[test]
fn flattens_stem_afferent_fields() {
    let flat = flatten_contract_event(&ContractEvent::StemAfferent(StemAfferentEvent {
        run_id: "run.1".to_string(),
        timestamp: "2026-03-28T10:00:00Z".to_string(),
        tick: 7,
        span_id: "stem.afferent:enqueue:sense-001".to_string(),
        parent_span_id: None,
        kind: "enqueue".to_string(),
        descriptor_id: "cli.input.text".to_string(),
        endpoint_id: Some("ep.cli".to_string()),
        sense_id: Some("sense:001".to_string()),
        sense_payload: Some(json!({"text": "hello"})),
        weight: Some(1.0),
        queue_state: Some(json!({"queue_name": "afferent"})),
        matched_rule_ids: None,
        reason: None,
    }));

    assert_eq!(flat.level, EventLevel::Info);
    assert_eq!(flat.family, "stem.afferent");
    assert_eq!(flat.direction.as_deref(), Some("afferent"));
    assert_eq!(flat.transition_kind.as_deref(), Some("enqueue"));
    assert_eq!(flat.endpoint_id.as_deref(), Some("ep.cli"));
    assert_eq!(flat.sense_id.as_deref(), Some("sense:001"));
    assert_eq!(flat.tick, 7);
}

#[test]
fn flattens_terminal_efferent_outcomes_as_warnings() {
    let flat = flatten_contract_event(&ContractEvent::StemEfferent(StemEfferentEvent {
        run_id: "run.1".to_string(),
        timestamp: "2026-03-28T10:00:00Z".to_string(),
        tick: 9,
        span_id: "stem.efferent:result:act-001".to_string(),
        parent_span_id: None,
        kind: "result".to_string(),
        act_id: "act:001".to_string(),
        descriptor_id: Some("shell.exec".to_string()),
        endpoint_id: Some("ep.shell".to_string()),
        act_payload: Some(json!({"command": "echo hello"})),
        queue_state: Some(json!({"queue_name": "efferent"})),
        continuity_decision: Some("continue".to_string()),
        terminal_outcome: Some(DispatchOutcomeClass::Lost),
        reason: Some(json!({"reason_code": "dispatch_lost"})),
    }));

    assert_eq!(flat.level, EventLevel::Warn);
    assert_eq!(flat.family, "stem.efferent");
    assert_eq!(flat.outcome.as_deref(), Some("lost"));
    assert_eq!(flat.act_id.as_deref(), Some("act:001"));
    assert_eq!(flat.tick, 9);
}
