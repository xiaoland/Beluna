use serde_json::json;

use super::flatten::{EventLevel, flatten_contract_event};
use crate::observability::contract::{
    ContractEvent, CortexOrganResponseEvent, DispatchOutcomeClass, OrganResponseStatus,
    SignalDirection, StemSignalTransitionEvent, TransitionKind,
};

#[test]
fn flattens_cortex_error_response_as_warning() {
    let flat = flatten_contract_event(&ContractEvent::CortexOrganResponse(
        CortexOrganResponseEvent {
            run_id: "run.1".to_string(),
            timestamp: "2026-03-28T10:00:00Z".to_string(),
            tick: 42,
            stage: "primary".to_string(),
            request_id: "req-1".to_string(),
            status: OrganResponseStatus::Error,
            response_summary: json!({"finish_reason": "error"}),
            tool_summary: json!([]),
            act_summary: json!([]),
            error_summary_when_present: Some(json!({"code": "timeout"})),
        },
    ));

    assert_eq!(flat.level, EventLevel::Warn);
    assert_eq!(flat.family, "cortex.organ.response");
    assert_eq!(flat.tick, Some(42));
    assert_eq!(flat.stage.as_deref(), Some("primary"));
    assert_eq!(flat.request_id.as_deref(), Some("req-1"));
    assert_eq!(flat.outcome.as_deref(), Some("error"));
}

#[test]
fn flattens_stem_signal_transition_fields() {
    let flat = flatten_contract_event(&ContractEvent::StemSignalTransition(
        StemSignalTransitionEvent {
            run_id: "run.1".to_string(),
            timestamp: "2026-03-28T10:00:00Z".to_string(),
            direction: SignalDirection::Afferent,
            transition_kind: TransitionKind::Enqueue,
            descriptor_id: "cli.input.text".to_string(),
            endpoint_id: Some("ep.cli".to_string()),
            sense_id: Some("sense:001".to_string()),
            act_id: None,
            tick_when_known: Some(7),
        },
    ));

    assert_eq!(flat.level, EventLevel::Info);
    assert_eq!(flat.family, "stem.signal.transition");
    assert_eq!(flat.direction.as_deref(), Some("afferent"));
    assert_eq!(flat.transition_kind.as_deref(), Some("enqueue"));
    assert_eq!(flat.endpoint_id.as_deref(), Some("ep.cli"));
    assert_eq!(flat.sense_id.as_deref(), Some("sense:001"));
    assert_eq!(flat.tick, Some(7));
}

#[test]
fn flattens_terminal_dispatch_outcomes_as_warnings() {
    let flat = flatten_contract_event(&ContractEvent::StemDispatchTransition(
        crate::observability::contract::StemDispatchTransitionEvent {
            run_id: "run.1".to_string(),
            timestamp: "2026-03-28T10:00:00Z".to_string(),
            act_id: "act:001".to_string(),
            transition_kind: TransitionKind::Result,
            queue_or_flow_summary: json!({"queue_name": "efferent"}),
            tick_when_known: Some(9),
            terminal_outcome_when_present: Some(DispatchOutcomeClass::Lost),
        },
    ));

    assert_eq!(flat.level, EventLevel::Warn);
    assert_eq!(flat.outcome.as_deref(), Some("lost"));
    assert_eq!(flat.act_id.as_deref(), Some("act:001"));
}
