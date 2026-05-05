use serde_json::{Value, json};

use crate::observability::{
    contract::{DispatchOutcomeClass, OrganResponseStatus},
    runtime::{AiGatewayChatTurnArgs, AiGatewayRequestArgs, current_run_id},
};

use super::{OwnerLogAttribute, OwnerLogEvent, OwnerLogSeverity, OwnerScope, emit};

pub fn emit_runtime_booted(config_path: String, signal_states: Value) {
    emit(OwnerLogEvent {
        scope: OwnerScope::Main,
        event_name: "runtime.booted",
        tick: 0,
        span_key: "boot".to_string(),
        severity: OwnerLogSeverity::Info,
        attributes: Vec::new(),
        body: json!({
            "summary": "Core runtime booted.",
            "run_id": current_run_id(),
            "config_path": config_path,
            "otlp_signal_states": signal_states,
        }),
    });
}

pub(crate) fn emit_tick_granted(tick: u64, tick_seq: u64) {
    emit(OwnerLogEvent {
        scope: OwnerScope::Stem,
        event_name: "tick.granted",
        tick,
        span_key: "grant".to_string(),
        severity: OwnerLogSeverity::Info,
        attributes: Vec::new(),
        body: json!({
            "summary": format!("Stem granted tick {tick_seq}."),
            "run_id": current_run_id(),
            "tick": tick,
            "tick_seq": tick_seq,
        }),
    });
}

pub(crate) fn emit_primary_started(route_or_backend: Option<&str>, tick: u64, input: Value) {
    emit(OwnerLogEvent {
        scope: OwnerScope::Cortex,
        event_name: "primary.started",
        tick,
        span_key: "primary".to_string(),
        severity: OwnerLogSeverity::Info,
        attributes: Vec::new(),
        body: json!({
            "summary": "Cortex primary phase started.",
            "organ_id": "primary",
            "route_or_backend": route_or_backend,
            "input": input,
        }),
    });
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn emit_primary_finished(
    tick: u64,
    status: OrganResponseStatus,
    output_payload: Option<Value>,
    error: Option<Value>,
    transport_request_id: Option<&str>,
    thread_id: Option<&str>,
    turn_id: Option<u64>,
) {
    emit(OwnerLogEvent {
        scope: OwnerScope::Cortex,
        event_name: "primary.finished",
        tick,
        span_key: "primary".to_string(),
        severity: severity_for_organ_status(status),
        attributes: Vec::new(),
        body: json!({
            "summary": "Cortex primary phase finished.",
            "organ_id": "primary",
            "status": status,
            "output": output_payload,
            "error": error,
            "transport_request_id": transport_request_id,
            "thread_id": thread_id,
            "turn_id": turn_id,
        }),
    });
}

pub(crate) fn emit_transport_request_completed(args: &AiGatewayRequestArgs) {
    if !matches!(args.kind.as_str(), "succeeded" | "failed") {
        return;
    }

    emit(OwnerLogEvent {
        scope: OwnerScope::AiGateway,
        event_name: "transport.request.completed",
        tick: args.tick,
        span_key: format!("request:{}", args.request_id),
        severity: severity_for_gateway_kind(&args.kind),
        attributes: vec![
            OwnerLogAttribute::string("ai.capability", args.capability.clone()),
            OwnerLogAttribute::string("ai.backend.id", args.backend_id.clone()),
            OwnerLogAttribute::string("ai.model", args.model.clone()),
        ],
        body: json!({
            "summary": "AI Gateway transport request completed.",
            "transport_request_id": args.request_id,
            "outcome": args.kind,
            "attempt": args.attempt,
            "retryable": args.retryable,
            "usage": args.usage,
            "error": args.error,
        }),
    });
}

pub(crate) fn emit_chat_turn_dispatched(args: &AiGatewayChatTurnArgs) {
    if args.status != "started" {
        return;
    }

    emit(OwnerLogEvent {
        scope: OwnerScope::AiGatewayChat,
        event_name: "turn.dispatched",
        tick: args.tick,
        span_key: chat_turn_span_key(&args.thread_id, args.turn_id),
        severity: OwnerLogSeverity::Info,
        attributes: Vec::new(),
        body: json!({
            "summary": "Chat turn dispatched to backend.",
            "chat_id": chat_id_from_metadata(&args.metadata),
            "thread_id": args.thread_id,
            "turn_id": args.turn_id,
            "transport_request_id": args.request_id,
            "dispatch_payload": args.dispatch_payload,
            "metadata": args.metadata,
        }),
    });
}

pub(crate) fn emit_chat_turn_committed(args: &AiGatewayChatTurnArgs) {
    let Some(messages_when_committed) = args.messages_when_committed.clone() else {
        return;
    };
    if args.error.is_some() {
        return;
    }

    emit(OwnerLogEvent {
        scope: OwnerScope::AiGatewayChat,
        event_name: "turn.committed",
        tick: args.tick,
        span_key: chat_turn_span_key(&args.thread_id, args.turn_id),
        severity: OwnerLogSeverity::Info,
        attributes: Vec::new(),
        body: json!({
            "summary": "Chat turn committed to thread.",
            "chat_id": chat_id_from_metadata(&args.metadata),
            "thread_id": args.thread_id,
            "turn_id": args.turn_id,
            "transport_request_id": args.request_id,
            "status": args.status,
            "messages_when_committed": messages_when_committed,
            "finish_reason": args.finish_reason,
            "usage": args.usage,
            "backend_metadata": args.backend_metadata,
        }),
    });
}

pub(crate) fn emit_act_delivered(
    tick: u64,
    act_id: &str,
    endpoint_id: Option<&str>,
    descriptor_id: Option<&str>,
    binding_kind: Option<&str>,
    act_payload: Option<Value>,
    outcome: DispatchOutcomeClass,
    reason_or_reference: Option<Value>,
) {
    if !matches!(outcome, DispatchOutcomeClass::Acknowledged) {
        return;
    }
    let (Some(endpoint_id), Some(descriptor_id)) = (endpoint_id, descriptor_id) else {
        return;
    };

    emit(OwnerLogEvent {
        scope: OwnerScope::Spine,
        event_name: "act.delivered",
        tick,
        span_key: format!("delivery:{act_id}"),
        severity: OwnerLogSeverity::Info,
        attributes: vec![
            OwnerLogAttribute::string("spine.act.id", act_id),
            OwnerLogAttribute::string("spine.endpoint.id", endpoint_id),
            OwnerLogAttribute::string("spine.descriptor.id", descriptor_id),
        ],
        body: json!({
            "summary": "Spine delivered act to endpoint.",
            "delivery": {
                "binding_kind": binding_kind,
                "acknowledged": true,
                "reference": reason_or_reference,
            },
            "act_payload": act_payload,
        }),
    });
}

fn severity_for_organ_status(status: OrganResponseStatus) -> OwnerLogSeverity {
    match status {
        OrganResponseStatus::Ok => OwnerLogSeverity::Info,
        OrganResponseStatus::Error => OwnerLogSeverity::Error,
    }
}

fn severity_for_gateway_kind(kind: &str) -> OwnerLogSeverity {
    match kind {
        "failed" => OwnerLogSeverity::Error,
        _ => OwnerLogSeverity::Info,
    }
}

fn chat_turn_span_key(thread_id: &str, turn_id: u64) -> String {
    format!("turn:{thread_id}:{turn_id}")
}

fn chat_id_from_metadata(metadata: &Value) -> Option<&str> {
    metadata.get("chat_id").and_then(Value::as_str)
}
