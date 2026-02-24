use std::collections::BTreeMap;

use crate::ai_gateway::types_chat::{
    BelunaMessage, BelunaToolDefinition, ChatResponse, OutputMode, RequestLimitOverrides,
    ToolChoice,
};

#[derive(Debug, Clone, Default)]
pub struct ChatSessionOpenRequest {
    pub session_id: Option<String>,
    pub default_route_ref: Option<String>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default)]
pub struct ChatThreadOpenRequest {
    pub thread_id: Option<String>,
    pub seed_messages: Vec<BelunaMessage>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ChatTurnRequest {
    pub request_id: Option<String>,
    pub route_ref_override: Option<String>,
    pub input_messages: Vec<BelunaMessage>,
    pub tools: Vec<BelunaToolDefinition>,
    pub tool_choice: ToolChoice,
    pub output_mode: OutputMode,
    pub limits: RequestLimitOverrides,
    pub metadata: BTreeMap<String, String>,
    pub cost_attribution_id: Option<String>,
}

impl Default for ChatTurnRequest {
    fn default() -> Self {
        Self {
            request_id: None,
            route_ref_override: None,
            input_messages: Vec::new(),
            tools: Vec::new(),
            tool_choice: ToolChoice::Auto,
            output_mode: OutputMode::Text,
            limits: RequestLimitOverrides::default(),
            metadata: BTreeMap::new(),
            cost_attribution_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChatTurnResponse {
    pub session_id: String,
    pub thread_id: String,
    pub turn_id: u64,
    pub response: ChatResponse,
}

#[derive(Debug, Clone)]
pub struct ChatThreadState {
    pub session_id: String,
    pub thread_id: String,
    pub next_turn_id: u64,
    pub message_count: usize,
    pub turns_total: u64,
    pub tool_calls_total: u64,
    pub failures_total: u64,
    pub last_turn_latency_ms: Option<u64>,
}
