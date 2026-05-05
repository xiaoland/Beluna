use serde_json::Value;

use crate::observability::owner_log;

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
    owner_log::events::emit_transport_request(&args);
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
    owner_log::events::emit_chat_turn_started(&args);
    owner_log::events::emit_chat_turn_finished(&args);
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
    pub source_thread_id: Option<String>,
    pub source_turn_ids: Option<Value>,
    pub kept_turn_ids: Option<Value>,
    pub dropped_turn_ids: Option<Value>,
    pub continuation_dropped: Option<bool>,
    pub context_reason: Option<String>,
}

pub fn emit_ai_gateway_chat_thread(args: AiGatewayChatThreadArgs) {
    owner_log::events::emit_chat_thread(&args);
}
