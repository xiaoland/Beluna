use serde_json::{Value, json};

use crate::observability::runtime::{
    AiGatewayChatThreadArgs, AiGatewayChatTurnArgs, AiGatewayRequestArgs, current_run_id,
};

mod spine;
mod stem;

pub(crate) use spine::{
    emit_act_bound, emit_act_outcome, emit_spine_adapter_lifecycle, emit_spine_endpoint_lifecycle,
    emit_spine_sense_ingress,
};
pub(crate) use stem::{
    emit_stem_afferent_pathway, emit_stem_afferent_rule, emit_stem_descriptor_catalog,
    emit_stem_efferent_pathway, emit_stem_proprioception,
};

use super::{
    OrganResponseStatus, OwnerLogAttribute, OwnerLogEvent, OwnerLogSeverity, OwnerScope, emit,
};

pub fn emit_runtime_booted(config_path: String, signal_states: Value) {
    emit(OwnerLogEvent {
        scope: OwnerScope::MainRuntime,
        event_name: "booted",
        tick: 0,
        span_key: "boot".to_string(),
        severity: OwnerLogSeverity::Info,
        attributes: Vec::new(),
        body: json!({
            "summary": "Core runtime booted.",
            "run_id": current_run_id(),
            "tick": 0,
            "config_path": config_path,
            "otlp_signal_states": signal_states,
        }),
    });
}

pub(crate) fn emit_tick_granted(tick: u64, tick_seq: u64) {
    emit(OwnerLogEvent {
        scope: OwnerScope::StemTick,
        event_name: "granted",
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

pub(crate) fn emit_cortex_organ_started(
    tick: u64,
    organ_id: &str,
    route_or_backend: Option<&str>,
    input: Value,
) {
    let Some(scope) = cortex_organ_scope(organ_id) else {
        return;
    };

    emit(OwnerLogEvent {
        scope,
        event_name: "started",
        tick,
        span_key: cortex_organ_span_key(organ_id),
        severity: OwnerLogSeverity::Info,
        attributes: Vec::new(),
        body: json!({
            "summary": format!("Cortex {organ_id} phase started."),
            "organ_id": organ_id,
            "route_or_backend": route_or_backend,
            "input": input,
        }),
    });
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn emit_cortex_organ_finished(
    tick: u64,
    organ_id: &str,
    status: OrganResponseStatus,
    output_payload: Option<Value>,
    error: Option<Value>,
    transport_request_id: Option<&str>,
    thread_id: Option<&str>,
    turn_id: Option<u64>,
) {
    let Some(scope) = cortex_organ_scope(organ_id) else {
        return;
    };

    emit(OwnerLogEvent {
        scope,
        event_name: "finished",
        tick,
        span_key: cortex_organ_span_key(organ_id),
        severity: severity_for_organ_status(status),
        attributes: Vec::new(),
        body: json!({
            "summary": format!("Cortex {organ_id} phase finished."),
            "organ_id": organ_id,
            "status": status,
            "output": output_payload,
            "error": error,
            "transport_request_id": transport_request_id,
            "thread_id": thread_id,
            "turn_id": turn_id,
        }),
    });
}

pub(crate) fn emit_transport_request(args: &AiGatewayRequestArgs) {
    let Some(event_name) = transport_request_event_name(&args.kind) else {
        return;
    };

    emit(OwnerLogEvent {
        scope: OwnerScope::AiGatewayTransport,
        event_name,
        tick: args.tick,
        span_key: format!("request:{}", args.request_id),
        severity: severity_for_gateway_kind(&args.kind),
        attributes: vec![
            OwnerLogAttribute::string("ai.capability", args.capability.clone()),
            OwnerLogAttribute::string("ai.backend.id", args.backend_id.clone()),
            OwnerLogAttribute::string("ai.model", args.model.clone()),
        ],
        body: json!({
            "summary": format!("AI Gateway transport request {event_name}."),
            "transport_request_id": args.request_id,
            "parent_span_id": args.parent_span_id,
            "organ_id": args.organ_id,
            "outcome": args.kind,
            "attempt": args.attempt,
            "retryable": args.retryable,
            "provider_request": args.provider_request,
            "provider_response": args.provider_response,
            "usage": args.usage,
            "error": args.error,
        }),
    });
}

pub(crate) fn emit_chat_turn_started(args: &AiGatewayChatTurnArgs) {
    if args.status != "started" {
        return;
    }

    emit(OwnerLogEvent {
        scope: OwnerScope::AiGatewayChat,
        event_name: "turn.started",
        tick: args.tick,
        span_key: chat_turn_span_key(&args.thread_id, args.turn_id),
        severity: OwnerLogSeverity::Info,
        attributes: Vec::new(),
        body: json!({
            "summary": "Chat turn started.",
            "chat_id": chat_id_from_metadata(&args.metadata),
            "thread_id": args.thread_id,
            "turn_id": args.turn_id,
            "parent_span_id": args.parent_span_id,
            "organ_id": args.organ_id,
            "transport_request_id": args.request_id,
            "turn_start_payload": args.dispatch_payload,
            "metadata": args.metadata,
        }),
    });
}

pub(crate) fn emit_chat_turn_finished(args: &AiGatewayChatTurnArgs) {
    if args.error.is_some() {
        emit_chat_turn_failed(args);
        return;
    }

    let Some(messages_when_committed) = args.messages_when_committed.clone() else {
        return;
    };

    emit(OwnerLogEvent {
        scope: OwnerScope::AiGatewayChat,
        event_name: "turn.finished",
        tick: args.tick,
        span_key: chat_turn_span_key(&args.thread_id, args.turn_id),
        severity: OwnerLogSeverity::Info,
        attributes: Vec::new(),
        body: json!({
            "summary": "Chat turn finished.",
            "chat_id": chat_id_from_metadata(&args.metadata),
            "thread_id": args.thread_id,
            "turn_id": args.turn_id,
            "parent_span_id": args.parent_span_id,
            "organ_id": args.organ_id,
            "transport_request_id": args.request_id,
            "status": args.status,
            "messages_when_committed": messages_when_committed,
            "finish_reason": args.finish_reason,
            "usage": args.usage,
            "backend_metadata": args.backend_metadata,
        }),
    });
}

fn emit_chat_turn_failed(args: &AiGatewayChatTurnArgs) {
    emit(OwnerLogEvent {
        scope: OwnerScope::AiGatewayChat,
        event_name: "turn.failed",
        tick: args.tick,
        span_key: chat_turn_span_key(&args.thread_id, args.turn_id),
        severity: OwnerLogSeverity::Error,
        attributes: Vec::new(),
        body: json!({
            "summary": "Chat turn failed.",
            "chat_id": chat_id_from_metadata(&args.metadata),
            "thread_id": args.thread_id,
            "turn_id": args.turn_id,
            "parent_span_id": args.parent_span_id,
            "organ_id": args.organ_id,
            "transport_request_id": args.request_id,
            "status": args.status,
            "turn_start_payload": args.dispatch_payload,
            "metadata": args.metadata,
            "backend_metadata": args.backend_metadata,
            "error": args.error,
        }),
    });
}

pub(crate) fn emit_chat_thread(args: &AiGatewayChatThreadArgs) {
    let event_name = chat_thread_event_name(&args.kind);

    emit(OwnerLogEvent {
        scope: OwnerScope::AiGatewayChat,
        event_name,
        tick: args.tick,
        span_key: format!("thread:{}", args.thread_id),
        severity: OwnerLogSeverity::Info,
        attributes: Vec::new(),
        body: json!({
            "summary": format!("Chat thread {event_name}."),
            "thread_id": args.thread_id,
            "thread_event_kind": args.kind,
            "parent_span_id": args.parent_span_id,
            "organ_id": args.organ_id,
            "transport_request_id": args.request_id,
            "messages": args.messages,
            "turn_summaries": args.turn_summaries,
            "source_thread_id": args.source_thread_id,
            "source_turn_ids": args.source_turn_ids,
            "kept_turn_ids": args.kept_turn_ids,
            "dropped_turn_ids": args.dropped_turn_ids,
            "continuation_dropped": args.continuation_dropped,
            "context_reason": args.context_reason,
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
        "attempt_failed" => OwnerLogSeverity::Warn,
        _ => OwnerLogSeverity::Info,
    }
}

fn transport_request_event_name(kind: &str) -> Option<&'static str> {
    match kind {
        "start" => Some("request.started"),
        "attempt_failed" => Some("attempt.failed"),
        "succeeded" => Some("request.finished"),
        "failed" => Some("request.failed"),
        _ => None,
    }
}

fn cortex_organ_scope(organ_id: &str) -> Option<OwnerScope> {
    match organ_id {
        "primary" => Some(OwnerScope::CortexPrimary),
        "attention" => Some(OwnerScope::CortexAttention),
        "cleanup" => Some(OwnerScope::CortexCleanup),
        "sense_helper" => Some(OwnerScope::CortexSenseHelper),
        "goal_forest_helper" => Some(OwnerScope::CortexGoalForest),
        "acts_helper" => Some(OwnerScope::CortexActsHelper),
        _ => None,
    }
}

fn cortex_organ_span_key(organ_id: &str) -> String {
    match organ_id {
        "sense_helper" => "sense-helper".to_string(),
        "goal_forest_helper" => "goal-forest".to_string(),
        "acts_helper" => "acts-helper".to_string(),
        _ => organ_id.to_string(),
    }
}

fn chat_thread_event_name(kind: &str) -> &'static str {
    match kind {
        "opened" => "thread.opened",
        "derived" => "thread.derived",
        "rewritten" => "thread.rewritten",
        _ => "thread.snapshot",
    }
}

fn chat_turn_span_key(thread_id: &str, turn_id: u64) -> String {
    format!("turn:{thread_id}:{turn_id}")
}

fn chat_id_from_metadata(metadata: &Value) -> Option<&str> {
    metadata.get("chat_id").and_then(Value::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_ai_gateway_kinds_to_native_event_names() {
        assert_eq!(
            transport_request_event_name("start"),
            Some("request.started")
        );
        assert_eq!(
            transport_request_event_name("attempt_failed"),
            Some("attempt.failed")
        );
        assert_eq!(
            transport_request_event_name("succeeded"),
            Some("request.finished")
        );
        assert_eq!(
            transport_request_event_name("failed"),
            Some("request.failed")
        );
        assert_eq!(chat_thread_event_name("opened"), "thread.opened");
        assert_eq!(chat_thread_event_name("turn_committed"), "thread.snapshot");
    }
}
