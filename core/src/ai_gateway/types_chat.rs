use std::{collections::BTreeMap, pin::Pin, sync::Arc};

use futures_core::Stream;
use serde::{Deserialize, Serialize};

use crate::ai_gateway::{
    error::GatewayError,
    types::{BackendDialect, BackendId, ModelId, RequestId},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BelunaRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BelunaContentPart {
    Text {
        text: String,
    },
    ImageUrl {
        url: String,
        mime_type: Option<String>,
    },
    Json {
        value: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BelunaMessage {
    pub role: BelunaRole,
    pub parts: Vec<BelunaContentPart>,
    #[serde(default)]
    pub tool_call_id: Option<String>,
    #[serde(default)]
    pub tool_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BelunaToolDefinition {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolChoice {
    Auto,
    None,
    Required,
    Specific { name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputMode {
    Text,
    JsonObject,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RequestLimitOverrides {
    #[serde(default)]
    pub max_output_tokens: Option<u64>,
    #[serde(default)]
    pub max_request_time_ms: Option<u64>,
}

fn default_tool_choice() -> ToolChoice {
    ToolChoice::Auto
}

fn default_output_mode() -> OutputMode {
    OutputMode::Text
}

#[derive(Debug, Clone)]
pub struct CanonicalRequest {
    pub request_id: RequestId,
    pub route_hint: Option<String>,
    pub messages: Vec<CanonicalMessage>,
    pub tools: Vec<CanonicalToolDefinition>,
    pub tool_choice: CanonicalToolChoice,
    pub output_mode: CanonicalOutputMode,
    pub limits: CanonicalLimits,
    pub metadata: BTreeMap<String, String>,
    pub cost_attribution_id: Option<String>,
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub route: Option<String>,
    pub messages: Vec<BelunaMessage>,
    #[serde(default)]
    pub tools: Vec<BelunaToolDefinition>,
    #[serde(default = "default_tool_choice")]
    pub tool_choice: ToolChoice,
    #[serde(default = "default_output_mode")]
    pub output_mode: OutputMode,
    #[serde(default)]
    pub limits: RequestLimitOverrides,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
    #[serde(default)]
    pub cost_attribution_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonicalRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone)]
pub struct CanonicalMessage {
    pub role: CanonicalRole,
    pub parts: Vec<CanonicalContentPart>,
    pub tool_call_id: Option<String>,
    pub tool_name: Option<String>,
}

#[derive(Debug, Clone)]
pub enum CanonicalContentPart {
    Text {
        text: String,
    },
    ImageUrl {
        url: String,
        mime_type: Option<String>,
    },
    Json {
        value: serde_json::Value,
    },
}

#[derive(Debug, Clone)]
pub struct CanonicalToolDefinition {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone)]
pub enum CanonicalToolChoice {
    Auto,
    None,
    Required,
    Specific { name: String },
}

#[derive(Debug, Clone)]
pub enum CanonicalOutputMode {
    Text,
    JsonObject,
}

#[derive(Debug, Clone, Default)]
pub struct CanonicalLimits {
    pub max_output_tokens: Option<u64>,
    pub max_request_time_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolCallStatus {
    Partial,
    Ready,
    Executed,
    Rejected,
}

#[derive(Debug, Clone)]
pub struct CanonicalToolCall {
    pub id: String,
    pub name: String,
    pub arguments_json: String,
    pub status: ToolCallStatus,
}

#[derive(Debug, Clone)]
pub struct UsageStats {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
    pub provider_usage_raw: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    Other(String),
}

#[derive(Debug, Clone)]
pub enum ChatEvent {
    Started {
        request_id: RequestId,
        backend_id: BackendId,
        model_id: ModelId,
    },
    TextDelta {
        request_id: RequestId,
        delta: String,
    },
    ToolCallDelta {
        request_id: RequestId,
        call_id: String,
        name: Option<String>,
        arguments_delta: String,
    },
    ToolCallReady {
        request_id: RequestId,
        call: CanonicalToolCall,
    },
    Usage {
        request_id: RequestId,
        usage: UsageStats,
    },
    Completed {
        request_id: RequestId,
        finish_reason: FinishReason,
    },
    Failed {
        request_id: RequestId,
        error: GatewayError,
    },
}

#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub request_id: RequestId,
    pub output_text: String,
    pub tool_calls: Vec<CanonicalToolCall>,
    pub usage: Option<UsageStats>,
    pub finish_reason: FinishReason,
    pub backend_metadata: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct BackendIdentity {
    pub backend_id: BackendId,
    pub dialect: BackendDialect,
    pub model: String,
}

#[derive(Debug, Clone)]
pub enum BackendRawEvent {
    OutputTextDelta {
        delta: String,
    },
    ToolCallDelta {
        call_id: String,
        name: Option<String>,
        arguments_delta: String,
    },
    ToolCallReady {
        call: CanonicalToolCall,
    },
    Usage {
        usage: UsageStats,
    },
    Completed {
        finish_reason: FinishReason,
    },
    Failed {
        error: GatewayError,
    },
}

pub type AdapterEventStream =
    Pin<Box<dyn Stream<Item = Result<BackendRawEvent, GatewayError>> + Send + 'static>>;
pub type AdapterCancelHandle = Arc<dyn Fn() + Send + Sync>;

pub struct AdapterInvocation {
    pub stream: AdapterEventStream,
    pub backend_identity: BackendIdentity,
    pub cancel: Option<AdapterCancelHandle>,
}

pub type ChatEventStream =
    Pin<Box<dyn Stream<Item = Result<ChatEvent, GatewayError>> + Send + 'static>>;
