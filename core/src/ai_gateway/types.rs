use std::{collections::BTreeMap, time::Duration};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

pub type BackendId = String;
pub type ModelId = String;
pub type RequestId = String;
pub const DEFAULT_ROUTE_ALIAS: &str = "default";
pub const CHAT_CAPABILITY_ID: &str = "chat";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ChatRouteAlias {
    #[validate(custom(function = "validate_non_blank"))]
    pub capability: String,
    #[validate(custom(function = "validate_non_blank"))]
    pub alias: String,
}

impl ChatRouteAlias {
    pub fn default_chat() -> Self {
        Self {
            capability: CHAT_CAPABILITY_ID.to_string(),
            alias: DEFAULT_ROUTE_ALIAS.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ChatRouteKey {
    #[validate(custom(function = "validate_non_blank"))]
    pub capability: String,
    #[validate(custom(function = "validate_non_blank"))]
    pub binding_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ChatRouteRef {
    Alias(ChatRouteAlias),
    Key(ChatRouteKey),
}

fn validate_non_blank(value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        return Err(ValidationError::new("non_blank"));
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema)]
pub enum BackendDialect {
    #[serde(rename = "openai_compatible")]
    OpenAiCompatible,
    #[serde(rename = "openai_responses")]
    OpenAiResponses,
    #[serde(rename = "ollama")]
    Ollama,
    #[serde(rename = "github_copilot_sdk")]
    GitHubCopilotSdk,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BackendCapabilities {
    #[serde(default)]
    pub streaming: bool,
    #[serde(default)]
    pub tool_calls: bool,
    #[serde(default)]
    pub parallel_tool_calls: bool,
    #[serde(default)]
    pub json_mode: bool,
    #[serde(default)]
    pub json_schema_mode: bool,
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
            parallel_tool_calls: false,
            json_mode: false,
            json_schema_mode: false,
            vision: false,
            resumable_streaming: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CopilotConfig {
    #[validate(custom(function = "validate_non_blank"))]
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BackendProfile {
    #[validate(custom(function = "validate_non_blank"))]
    pub id: BackendId,
    pub dialect: BackendDialect,
    #[validate(custom(function = "validate_non_blank"))]
    pub endpoint: Option<String>,
    pub credential: CredentialRef,
    #[validate(length(min = 1))]
    #[validate(nested)]
    pub models: Vec<ModelProfile>,
    #[serde(default)]
    #[validate(nested)]
    pub capabilities: Option<BackendCapabilities>,
    #[serde(default)]
    #[validate(nested)]
    pub copilot: Option<CopilotConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ModelProfile {
    #[validate(custom(function = "validate_non_blank"))]
    pub id: ModelId,
    #[serde(default)]
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelTarget {
    pub backend_id: BackendId,
    pub model_id: ModelId,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResilienceConfig {
    #[serde(default = "default_max_request_time_ms")]
    #[validate(range(min = 1))]
    pub max_request_time_ms: u64,
    #[serde(default = "default_request_timeout_ms")]
    #[validate(range(min = 1))]
    pub request_timeout_ms: u64,
    #[serde(default = "default_max_retries")]
    #[validate(range(min = 0))]
    pub max_retries: u32,
    #[serde(default = "default_backoff_base_ms")]
    #[validate(range(min = 1))]
    pub backoff_base_ms: u64,
    #[serde(default = "default_backoff_max_ms")]
    #[validate(range(min = 1))]
    pub backoff_max_ms: u64,
    #[serde(default)]
    pub retry_policy: RetryPolicy,
    #[serde(default = "default_breaker_failure_threshold")]
    #[validate(range(min = 1))]
    pub breaker_failure_threshold: u32,
    #[serde(default = "default_breaker_open_ms")]
    #[validate(range(min = 1))]
    pub breaker_open_ms: u64,
    #[serde(default = "default_max_concurrency_per_backend")]
    #[validate(range(min = 1))]
    pub max_concurrency_per_backend: u32,
    #[serde(default)]
    #[validate(range(min = 1))]
    pub rate_smoothing_per_second: Option<u32>,
}

impl Default for ResilienceConfig {
    fn default() -> Self {
        Self {
            max_request_time_ms: default_max_request_time_ms(),
            request_timeout_ms: default_request_timeout_ms(),
            max_retries: default_max_retries(),
            backoff_base_ms: default_backoff_base_ms(),
            backoff_max_ms: default_backoff_max_ms(),
            retry_policy: RetryPolicy::BeforeFirstEventOnly,
            breaker_failure_threshold: default_breaker_failure_threshold(),
            breaker_open_ms: default_breaker_open_ms(),
            max_concurrency_per_backend: default_max_concurrency_per_backend(),
            rate_smoothing_per_second: None,
        }
    }
}

fn default_chat_default_route() -> Option<ChatRouteRef> {
    Some(ChatRouteRef::Alias(ChatRouteAlias::default_chat()))
}

fn default_chat_default_max_tool_rounds() -> u32 {
    6
}

fn default_chat_default_max_turn_context_messages() -> usize {
    64
}

fn default_chat_default_session_ttl_seconds() -> u64 {
    3_600
}

fn default_chat_default_turn_timeout_ms() -> u64 {
    30_000
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ChatConfig {
    #[serde(default = "default_chat_default_route")]
    pub default_route: Option<ChatRouteRef>,
    #[serde(default = "default_chat_default_max_tool_rounds")]
    #[validate(range(min = 1))]
    pub default_max_tool_rounds: u32,
    #[serde(default = "default_chat_default_max_turn_context_messages")]
    #[validate(range(min = 1))]
    pub default_max_turn_context_messages: usize,
    #[serde(default = "default_chat_default_session_ttl_seconds")]
    #[validate(range(min = 1))]
    pub default_session_ttl_seconds: u64,
    #[serde(default = "default_chat_default_turn_timeout_ms")]
    #[validate(range(min = 1))]
    pub default_turn_timeout_ms: u64,
}

impl Default for ChatConfig {
    fn default() -> Self {
        Self {
            default_route: default_chat_default_route(),
            default_max_tool_rounds: default_chat_default_max_tool_rounds(),
            default_max_turn_context_messages: default_chat_default_max_turn_context_messages(),
            default_session_ttl_seconds: default_chat_default_session_ttl_seconds(),
            default_turn_timeout_ms: default_chat_default_turn_timeout_ms(),
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AIGatewayConfig {
    #[validate(length(min = 1))]
    #[validate(nested)]
    pub backends: Vec<BackendProfile>,
    #[serde(default)]
    #[validate(nested)]
    pub chat: ChatConfig,
    #[serde(default)]
    #[validate(nested)]
    pub resilience: ResilienceConfig,
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
