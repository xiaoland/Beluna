use std::{collections::BTreeMap, pin::Pin, sync::Arc, time::Duration};

use futures_core::Stream;
use serde::{Deserialize, Serialize};

use crate::ai_gateway::error::GatewayError;

pub type BackendId = String;
pub type RequestId = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BackendDialect {
    #[serde(rename = "openai_compatible")]
    OpenAiCompatible,
    #[serde(rename = "ollama")]
    Ollama,
    #[serde(rename = "github_copilot_sdk")]
    GitHubCopilotSdk,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackendCapabilities {
    #[serde(default)]
    pub streaming: bool,
    #[serde(default)]
    pub tool_calls: bool,
    #[serde(default)]
    pub json_mode: bool,
    #[serde(default)]
    pub vision: bool,
    #[serde(default)]
    pub resumable_streaming: bool,
}

impl Default for BackendCapabilities {
    fn default() -> Self {
        Self {
            streaming: true,
            tool_calls: false,
            json_mode: false,
            vision: false,
            resumable_streaming: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CredentialRef {
    Env { var: String },
    InlineToken { token: String },
    None,
}

#[derive(Debug, Clone)]
pub struct ResolvedCredential {
    pub auth_header: Option<String>,
    pub extra_headers: Vec<(String, String)>,
    pub opaque: BTreeMap<String, String>,
}

impl ResolvedCredential {
    pub fn none() -> Self {
        Self {
            auth_header: None,
            extra_headers: Vec::new(),
            opaque: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendProfile {
    pub id: BackendId,
    pub dialect: BackendDialect,
    pub endpoint: Option<String>,
    pub credential: CredentialRef,
    pub default_model: String,
    #[serde(default)]
    pub capabilities: Option<BackendCapabilities>,
    #[serde(default)]
    pub copilot: Option<CopilotConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityConfig {
    #[serde(default = "default_request_timeout_ms")]
    pub request_timeout_ms: u64,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default = "default_backoff_base_ms")]
    pub backoff_base_ms: u64,
    #[serde(default = "default_backoff_max_ms")]
    pub backoff_max_ms: u64,
    #[serde(default)]
    pub retry_policy: RetryPolicy,
    #[serde(default = "default_breaker_failure_threshold")]
    pub breaker_failure_threshold: u32,
    #[serde(default = "default_breaker_open_ms")]
    pub breaker_open_ms: u64,
}

impl Default for ReliabilityConfig {
    fn default() -> Self {
        Self {
            request_timeout_ms: default_request_timeout_ms(),
            max_retries: default_max_retries(),
            backoff_base_ms: default_backoff_base_ms(),
            backoff_max_ms: default_backoff_max_ms(),
            retry_policy: RetryPolicy::BeforeFirstEventOnly,
            breaker_failure_threshold: default_breaker_failure_threshold(),
            breaker_open_ms: default_breaker_open_ms(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    #[serde(default = "default_max_request_time_ms")]
    pub max_request_time_ms: u64,
    #[serde(default)]
    pub max_usage_tokens_per_request: Option<u64>,
    #[serde(default = "default_max_concurrency_per_backend")]
    pub max_concurrency_per_backend: u32,
    #[serde(default)]
    pub rate_smoothing_per_second: Option<u32>,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            max_request_time_ms: default_max_request_time_ms(),
            max_usage_tokens_per_request: None,
            max_concurrency_per_backend: default_max_concurrency_per_backend(),
            rate_smoothing_per_second: None,
        }
    }
}

fn default_request_timeout_ms() -> u64 {
    30_000
}

fn default_max_retries() -> u32 {
    2
}

fn default_backoff_base_ms() -> u64 {
    200
}

fn default_backoff_max_ms() -> u64 {
    2_000
}

fn default_breaker_failure_threshold() -> u32 {
    5
}

fn default_breaker_open_ms() -> u64 {
    15_000
}

fn default_max_request_time_ms() -> u64 {
    45_000
}

fn default_max_concurrency_per_backend() -> u32 {
    8
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetryPolicy {
    BeforeFirstEventOnly,
    AdapterResumable,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::BeforeFirstEventOnly
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIGatewayConfig {
    pub default_backend: BackendId,
    pub backends: Vec<BackendProfile>,
    #[serde(default)]
    pub reliability: ReliabilityConfig,
    #[serde(default)]
    pub budget: BudgetConfig,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BelunaInferenceRequest {
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub backend_id: Option<BackendId>,
    #[serde(default)]
    pub model: Option<String>,
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
    #[serde(default = "default_stream")]
    pub stream: bool,
}

fn default_tool_choice() -> ToolChoice {
    ToolChoice::Auto
}

fn default_output_mode() -> OutputMode {
    OutputMode::Text
}

fn default_stream() -> bool {
    true
}

#[derive(Debug, Clone)]
pub struct CanonicalRequest {
    pub request_id: RequestId,
    pub backend_hint: Option<BackendId>,
    pub model_override: Option<String>,
    pub messages: Vec<CanonicalMessage>,
    pub tools: Vec<CanonicalToolDefinition>,
    pub tool_choice: CanonicalToolChoice,
    pub output_mode: CanonicalOutputMode,
    pub limits: CanonicalLimits,
    pub metadata: BTreeMap<String, String>,
    pub stream: bool,
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
pub enum GatewayEvent {
    Started {
        request_id: RequestId,
        backend_id: BackendId,
        model: String,
    },
    OutputTextDelta {
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
pub struct CanonicalFinalResponse {
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

#[derive(Debug, Clone)]
pub struct AdapterContext {
    pub backend_id: BackendId,
    pub model: String,
    pub profile: BackendProfile,
    pub credential: ResolvedCredential,
    pub timeout: Duration,
    pub request_id: RequestId,
}

pub type GatewayEventStream =
    Pin<Box<dyn Stream<Item = Result<GatewayEvent, GatewayError>> + Send + 'static>>;
