# L2-05 - Config Schema and Test Plan

- Task Name: `minimal-ai-gateway`
- Stage: `L2` detail: config and tests
- Date: 2026-02-08
- Status: `DRAFT_FOR_APPROVAL`

## 1) Rust Config Struct Additions

```rust
pub struct Config {
    pub socket_path: PathBuf,
    pub ai_gateway: AIGatewayConfig,
}

pub struct AIGatewayConfig {
    pub default_backend: BackendId,
    pub backends: Vec<BackendProfile>,
    pub reliability: ReliabilityConfig,
    pub budget: BudgetConfig,
}

pub struct BackendProfile {
    pub id: BackendId,
    pub dialect: BackendDialect,
    pub endpoint: Option<String>,
    pub credential: CredentialRef,
    pub default_model: String,
    pub capabilities: Option<BackendCapabilities>,
    pub copilot: Option<CopilotConfig>,
}

pub enum BackendDialect {
    OpenAiCompatible,
    Ollama,
    GitHubCopilotSdk,
}

#[serde(tag = "type", rename_all = "snake_case")]
pub enum CredentialRef {
    Env { var: String },
    InlineToken { token: String },
    None,
}

pub struct ReliabilityConfig {
    pub request_timeout_ms: u64,
    pub max_retries: u32,
    pub backoff_base_ms: u64,
    pub backoff_max_ms: u64,
    pub retry_policy: RetryPolicy,
    pub breaker_failure_threshold: u32,
    pub breaker_open_ms: u64,
}

pub struct BudgetConfig {
    pub max_request_time_ms: u64,
    pub max_usage_tokens_per_request: Option<u64>,
    pub max_concurrency_per_backend: u32,
    pub rate_smoothing_per_second: Option<u32>,
}
```

## 2) JSON Schema Additions (Design Targets)

`beluna.schema.json` adds:

1. top-level `ai_gateway` object (required for gateway-enabled runtime)
2. `backends[]` with strict `dialect` enum:
- `openai_compatible`
- `ollama`
- `github_copilot_sdk`
3. `credential` uses tagged object shape with required `type`:
- `{ "type": "env", "var": "..." }`
- `{ "type": "inline_token", "token": "..." }`
- `{ "type": "none" }`
4. strict validation with `additionalProperties: false` on all nested objects
5. numeric minimum constraints:
- timeouts/backoffs > 0
- retries >= 0
- concurrency >= 1

## 3) Example Config Sketch

```jsonc
{
  "socket_path": "/tmp/beluna.sock",
  "ai_gateway": {
    "default_backend": "openai-default",
    "reliability": {
      "request_timeout_ms": 30000,
      "max_retries": 2,
      "backoff_base_ms": 200,
      "backoff_max_ms": 2000,
      "retry_policy": "before_first_event_only",
      "breaker_failure_threshold": 5,
      "breaker_open_ms": 15000
    },
    "budget": {
      "max_request_time_ms": 45000,
      "max_usage_tokens_per_request": 16000,
      "max_concurrency_per_backend": 8,
      "rate_smoothing_per_second": 20
    },
    "backends": [
      {
        "id": "openai-default",
        "dialect": "openai_compatible",
        "endpoint": "https://api.openai.com/v1",
        "credential": { "type": "env", "var": "OPENAI_API_KEY" },
        "default_model": "gpt-4.1-mini"
      },
      {
        "id": "ollama-local",
        "dialect": "ollama",
        "endpoint": "http://127.0.0.1:11434",
        "credential": { "type": "none" },
        "default_model": "qwen2.5-coder:7b"
      },
      {
        "id": "copilot",
        "dialect": "github_copilot_sdk",
        "credential": { "type": "env", "var": "GITHUB_TOKEN" },
        "default_model": "copilot-default",
        "copilot": {
          "command": "copilot-language-server",
          "args": ["--stdio"]
        }
      }
    ]
  }
}
```

## 4) Unit Test Matrix

1. RequestNormalizer
- rejects empty messages
- generates request ID when missing
- strict tool schema validation
- enforces tool-message invariants:
- role=`tool` requires `tool_call_id`, requires `tool_name`, rejects `ImageUrl` parts
- role!=`tool` requires `tool_call_id=None` and `tool_name=None`

2. BackendRouter
- deterministic backend selection (request override else default only)
- no multi-backend fallback when selected backend fails

3. CapabilityGuard
- rejects unsupported vision/json/tool_calls per backend
- accepts override-enabled capabilities

4. ReliabilityLayer
- retries only before first event
- no retry after first event by default
- no retry after tool-call event unless marked safe
- cancels in-flight adapter request when consumer drops stream

5. CircuitBreaker
- opens after threshold failures
- rejects during open window
- probe behavior after open window

6. BudgetEnforcer
- concurrency limit blocks excess requests
- timeout budget cancels long-running invocation
- usage-token post-check is best-effort accounting only and does not terminate active stream

7. Error mapper
- per-dialect status/code -> canonical kind mapping

## 5) Integration Test Matrix

1. OpenAI-compatible mock server
- streaming SSE -> canonical events ordering
- tool-call delta assembly
- usage extraction from terminal chunk

2. Ollama mock server
- NDJSON streaming parse and completion
- usage extraction fields mapping

3. Copilot mock process
- JSON-RPC framing and init handshake
- auth failure mapping
- stream event normalization

4. End-to-end gateway
- backend selection + credentials + reliability + budget + telemetry

5. Stream cancellation
- dropping consumer stream aborts HTTP/Copilot in-flight work and releases permits

## 6) Non-goals for MVP Tests

- real provider live-network tests in CI
- cost estimation accuracy tests
- resumable-streaming retry behavior (future capability)

## 7) L2 Acceptance Checklist

L2 is considered approved when:

1. module boundaries are accepted,
2. canonical event model is accepted,
3. retry safety semantics are accepted,
4. backend adapter mappings are accepted,
5. config + test scope are accepted.

Status: `READY_FOR_L2_REVIEW`
