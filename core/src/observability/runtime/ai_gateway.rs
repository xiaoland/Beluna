use serde_json::Value;

use crate::observability::contract::{
    AiGatewayChatThreadEvent, AiGatewayChatTurnEvent, AiGatewayRequestEvent, ContractEvent,
};

use super::{current_run_id, emit_contract_event, timestamp_now};

pub struct AiGatewayRequestArgs {
    pub tick: u64,
    pub request_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub organ_id: Option<String>,
    pub capability: String,
    pub backend_id: String,
    pub model: String,
    pub kind: String,
    pub attempt: Option<u32>,
    pub retryable: Option<bool>,
    pub provider_request: Option<Value>,
    pub provider_response: Option<Value>,
    pub usage: Option<Value>,
    pub error: Option<Value>,
}

pub fn emit_ai_gateway_request(args: AiGatewayRequestArgs) {
    emit_contract_event(ContractEvent::AiGatewayRequest(AiGatewayRequestEvent {
        run_id: current_run_id().to_string(),
        timestamp: timestamp_now(),
        tick: args.tick,
        request_id: args.request_id,
        span_id: args.span_id,
        parent_span_id: args.parent_span_id,
        organ_id: args.organ_id,
        capability: args.capability,
        backend_id: args.backend_id,
        model: args.model,
        kind: args.kind,
        attempt: args.attempt,
        retryable: args.retryable,
        provider_request: args.provider_request,
        provider_response: args.provider_response,
        usage: args.usage,
        error: args.error,
    }));
}

pub struct AiGatewayChatTurnArgs {
    pub tick: u64,
    pub thread_id: String,
    pub turn_id: u64,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub organ_id: Option<String>,
    pub request_id: Option<String>,
    pub status: String,
    pub dispatch_payload: Value,
    pub messages_when_committed: Option<Value>,
    pub metadata: Value,
    pub finish_reason: Option<Value>,
    pub usage: Option<Value>,
    pub backend_metadata: Option<Value>,
    pub error: Option<Value>,
}

pub fn emit_ai_gateway_chat_turn(args: AiGatewayChatTurnArgs) {
    emit_contract_event(ContractEvent::AiGatewayChatTurn(AiGatewayChatTurnEvent {
        run_id: current_run_id().to_string(),
        timestamp: timestamp_now(),
        tick: args.tick,
        thread_id: args.thread_id,
        turn_id: args.turn_id,
        span_id: args.span_id,
        parent_span_id: args.parent_span_id,
        organ_id: args.organ_id,
        request_id: args.request_id,
        status: args.status,
        dispatch_payload: args.dispatch_payload,
        messages_when_committed: args.messages_when_committed,
        metadata: args.metadata,
        finish_reason: args.finish_reason,
        usage: args.usage,
        backend_metadata: args.backend_metadata,
        error: args.error,
    }));
}

pub struct AiGatewayChatThreadArgs {
    pub tick: u64,
    pub thread_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub organ_id: Option<String>,
    pub request_id: Option<String>,
    pub kind: String,
    pub messages: Value,
    pub turn_summaries: Option<Value>,
    pub source_turn_ids: Option<Value>,
}

pub fn emit_ai_gateway_chat_thread(args: AiGatewayChatThreadArgs) {
    emit_contract_event(ContractEvent::AiGatewayChatThread(
        AiGatewayChatThreadEvent {
            run_id: current_run_id().to_string(),
            timestamp: timestamp_now(),
            tick: args.tick,
            thread_id: args.thread_id,
            span_id: args.span_id,
            parent_span_id: args.parent_span_id,
            organ_id: args.organ_id,
            request_id: args.request_id,
            kind: args.kind,
            messages: args.messages,
            turn_summaries: args.turn_summaries,
            source_turn_ids: args.source_turn_ids,
        },
    ));
}
