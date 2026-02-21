use std::{collections::BTreeMap, time::Duration};

use serde::{Deserialize, Serialize};

pub type BackendId = String;
pub type ModelId = String;
pub type RequestId = String;
pub const DEFAULT_ROUTE_ALIAS: &str = "default";

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
            json_mode: false,
            json_schema_mode: false,
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
    pub models: Vec<ModelProfile>,
    #[serde(default)]
    pub capabilities: Option<BackendCapabilities>,
    #[serde(default)]
    pub copilot: Option<CopilotConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelProfile {
    pub id: ModelId,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelTarget {
    pub backend_id: BackendId,
    pub model_id: ModelId,
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
    pub backends: Vec<BackendProfile>,
    #[serde(default)]
    pub route_aliases: BTreeMap<String, ModelTarget>,
    #[serde(default)]
    pub reliability: ReliabilityConfig,
    #[serde(default)]
    pub budget: BudgetConfig,
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
