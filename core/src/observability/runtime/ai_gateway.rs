use serde_json::Value;

use crate::observability::contract::{
    AiGatewayRequestEvent, AiGatewayThreadEvent, AiGatewayTurnEvent, ContractEvent,
};

use super::{current_run_id, emit_contract_event, timestamp_now};

pub struct AiGatewayRequestArgs {
    pub tick: u64,
    pub request_id: String,
    pub span_id: String,
    pub parent_span_id_when_present: Option<String>,
    pub organ_id_when_present: Option<String>,
    pub thread_id_when_present: Option<String>,
    pub turn_id_when_present: Option<u64>,
    pub backend_id: String,
    pub model: String,
    pub kind: String,
    pub attempt_when_present: Option<u32>,
    pub input_payload: Value,
    pub effective_tools_when_present: Option<Value>,
    pub limits_when_present: Option<Value>,
    pub enable_thinking: bool,
    pub provider_request_when_present: Option<Value>,
    pub provider_response_when_present: Option<Value>,
    pub usage_when_present: Option<Value>,
    pub error_when_present: Option<Value>,
}

pub fn emit_ai_gateway_request(args: AiGatewayRequestArgs) {
    emit_contract_event(ContractEvent::AiGatewayRequest(AiGatewayRequestEvent {
        run_id: current_run_id().to_string(),
        timestamp: timestamp_now(),
        tick: args.tick,
        request_id: args.request_id,
        span_id: args.span_id,
        parent_span_id_when_present: args.parent_span_id_when_present,
        organ_id_when_present: args.organ_id_when_present,
        thread_id_when_present: args.thread_id_when_present,
        turn_id_when_present: args.turn_id_when_present,
        backend_id: args.backend_id,
        model: args.model,
        kind: args.kind,
        attempt_when_present: args.attempt_when_present,
        input_payload: args.input_payload,
        effective_tools_when_present: args.effective_tools_when_present,
        limits_when_present: args.limits_when_present,
        enable_thinking: args.enable_thinking,
        provider_request_when_present: args.provider_request_when_present,
        provider_response_when_present: args.provider_response_when_present,
        usage_when_present: args.usage_when_present,
        error_when_present: args.error_when_present,
    }));
}

pub struct AiGatewayTurnArgs {
    pub tick: u64,
    pub thread_id: String,
    pub turn_id: u64,
    pub span_id: String,
    pub parent_span_id_when_present: Option<String>,
    pub organ_id_when_present: Option<String>,
    pub request_id_when_present: Option<String>,
    pub status: String,
    pub messages_when_committed: Option<Value>,
    pub metadata: Value,
    pub finish_reason_when_present: Option<Value>,
    pub usage_when_present: Option<Value>,
    pub backend_metadata_when_present: Option<Value>,
    pub error_when_present: Option<Value>,
}

pub fn emit_ai_gateway_turn(args: AiGatewayTurnArgs) {
    emit_contract_event(ContractEvent::AiGatewayTurn(AiGatewayTurnEvent {
        run_id: current_run_id().to_string(),
        timestamp: timestamp_now(),
        tick: args.tick,
        thread_id: args.thread_id,
        turn_id: args.turn_id,
        span_id: args.span_id,
        parent_span_id_when_present: args.parent_span_id_when_present,
        organ_id_when_present: args.organ_id_when_present,
        request_id_when_present: args.request_id_when_present,
        status: args.status,
        messages_when_committed: args.messages_when_committed,
        metadata: args.metadata,
        finish_reason_when_present: args.finish_reason_when_present,
        usage_when_present: args.usage_when_present,
        backend_metadata_when_present: args.backend_metadata_when_present,
        error_when_present: args.error_when_present,
    }));
}

pub struct AiGatewayThreadArgs {
    pub tick: u64,
    pub thread_id: String,
    pub span_id: String,
    pub parent_span_id_when_present: Option<String>,
    pub organ_id_when_present: Option<String>,
    pub kind: String,
    pub messages: Value,
    pub turn_summaries_when_present: Option<Value>,
    pub source_turn_ids_when_present: Option<Value>,
}

pub fn emit_ai_gateway_thread(args: AiGatewayThreadArgs) {
    emit_contract_event(ContractEvent::AiGatewayThread(AiGatewayThreadEvent {
        run_id: current_run_id().to_string(),
        timestamp: timestamp_now(),
        tick: args.tick,
        thread_id: args.thread_id,
        span_id: args.span_id,
        parent_span_id_when_present: args.parent_span_id_when_present,
        organ_id_when_present: args.organ_id_when_present,
        kind: args.kind,
        messages: args.messages,
        turn_summaries_when_present: args.turn_summaries_when_present,
        source_turn_ids_when_present: args.source_turn_ids_when_present,
    }));
}
