use std::{collections::BTreeMap, pin::Pin, sync::Arc};

use futures_core::Stream;
use serde::{Deserialize, Serialize};

use crate::ai_gateway::{
    error::GatewayError,
    types::{BackendDialect, BackendId, ModelId},
};

use super::tool::ChatToolDefinition;

// ---------------------------------------------------------------------------
// Chat-domain primitives
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChatRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentPart {
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
pub struct ChatMessage {
    pub role: ChatRole,
    pub parts: Vec<ContentPart>,
    #[serde(default)]
    pub tool_call_id: Option<String>,
    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub tool_calls: Vec<MessageToolCall>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageToolCall {
    pub id: String,
    pub name: String,
    pub arguments_json: String,
}

// ---------------------------------------------------------------------------
// Output mode
// ---------------------------------------------------------------------------

fn default_json_schema_strict_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputMode {
    Text,
    JsonObject,
    JsonSchema {
        name: String,
        schema: serde_json::Value,
        #[serde(default = "default_json_schema_strict_true")]
        strict: bool,
    },
}

// ---------------------------------------------------------------------------
// Limits
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TurnLimits {
    #[serde(default)]
    pub max_output_tokens: Option<u64>,
    #[serde(default)]
    pub max_request_time_ms: Option<u64>,
}

// ---------------------------------------------------------------------------
// Tool-call results returned by backends
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolCallStatus {
    Partial,
    Ready,
    Executed,
    Rejected,
}

#[derive(Debug, Clone)]
pub struct ToolCallResult {
    pub id: String,
    pub name: String,
    pub arguments_json: String,
    pub status: ToolCallStatus,
}

// ---------------------------------------------------------------------------
// Usage
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct UsageStats {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
    pub provider_usage_raw: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Finish reason
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    Other(String),
}

// ---------------------------------------------------------------------------
// TurnPayload — what the dispatcher / adapters receive (internal)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub(crate) struct TurnPayload {
    pub messages: Arc<Vec<ChatMessage>>,
    pub tools: Vec<ChatToolDefinition>,
    pub output_mode: OutputMode,
    pub limits: TurnLimits,
    pub enable_thinking: bool,
    pub metadata: BTreeMap<String, String>,
}

// ---------------------------------------------------------------------------
// Turn response
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TurnResponse {
    pub output_text: String,
    pub tool_calls: Vec<ToolCallResult>,
    pub usage: Option<UsageStats>,
    pub finish_reason: FinishReason,
    pub backend_metadata: BTreeMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Adapter-level types (backend → dispatcher)
// ---------------------------------------------------------------------------

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
        call: ToolCallResult,
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

#[derive(Debug, Clone)]
pub struct BackendCompleteResponse {
    pub backend_identity: BackendIdentity,
    pub output_text: String,
    pub tool_calls: Vec<ToolCallResult>,
    pub usage: Option<UsageStats>,
    pub finish_reason: FinishReason,
}

// ---------------------------------------------------------------------------
// Streaming events (dispatcher → caller)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum ChatEvent {
    Started {
        backend_id: BackendId,
        model_id: ModelId,
    },
    TextDelta {
        delta: String,
    },
    ToolCallDelta {
        call_id: String,
        name: Option<String>,
        arguments_delta: String,
    },
    ToolCallReady {
        call: ToolCallResult,
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

impl ChatEvent {
    pub fn is_output(&self) -> bool {
        matches!(
            self,
            ChatEvent::TextDelta { .. }
                | ChatEvent::ToolCallDelta { .. }
                | ChatEvent::ToolCallReady { .. }
        )
    }

    pub fn is_tool(&self) -> bool {
        matches!(
            self,
            ChatEvent::ToolCallDelta { .. } | ChatEvent::ToolCallReady { .. }
        )
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, ChatEvent::Completed { .. } | ChatEvent::Failed { .. })
    }
}

pub type ChatEventStream =
    Pin<Box<dyn Stream<Item = Result<ChatEvent, GatewayError>> + Send + 'static>>;
