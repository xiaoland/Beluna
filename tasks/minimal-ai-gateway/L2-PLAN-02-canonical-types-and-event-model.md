# L2-02 - Canonical Types and Event Model

- Task Name: `minimal-ai-gateway`
- Stage: `L2` detail: canonical model
- Date: 2026-02-08
- Status: `DRAFT_FOR_APPROVAL`

## 1) Canonical Request Model

```rust
pub struct BelunaInferenceRequest {
    pub request_id: Option<String>,
    pub backend_id: Option<BackendId>,
    pub model: Option<String>,
    pub messages: Vec<BelunaMessage>,
    pub tools: Vec<BelunaToolDefinition>,
    pub tool_choice: ToolChoice,
    pub output_mode: OutputMode,
    pub limits: RequestLimitOverrides,
    pub metadata: BTreeMap<String, String>,
    pub stream: bool,
}

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
```

Normalization rules:

1. If `request_id` missing, generate UUIDv7.
2. Empty message list is invalid.
3. Unknown tool schema keywords are rejected (strict mode).
4. `output_mode=json` is validated by `CapabilityGuard` after backend selection.
5. Message role/linkage invariants are validated by `RequestNormalizer` for deterministic `InvalidRequest` errors.

## 2) Canonical Message and Content Parts

```rust
pub enum CanonicalRole {
    System,
    User,
    Assistant,
    Tool,
}

pub struct CanonicalMessage {
    pub role: CanonicalRole,
    pub parts: Vec<CanonicalContentPart>,
    pub tool_call_id: Option<String>,
    pub tool_name: Option<String>,
}

pub enum CanonicalContentPart {
    Text { text: String },
    ImageUrl { url: String, mime_type: Option<String> },
    Json { value: serde_json::Value },
}
```

MVP constraint:
- Only `Text` is required for all backends.
- `ImageUrl` accepted only when backend capability `vision=true`.

Message role/linkage invariants (MVP strict):

1. If `role == Tool`:
- `tool_call_id` must exist.
- `tool_name` must exist.
- `parts` may only contain `Text` or `Json` (no `ImageUrl`).
2. If `role != Tool`:
- `tool_call_id` must be `None`.
- `tool_name` must be `None`.

## 3) Canonical Tool Model

```rust
pub struct CanonicalToolDefinition {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
}

pub enum CanonicalToolChoice {
    Auto,
    None,
    Required,
    Specific { name: String },
}

pub struct CanonicalToolCall {
    pub id: String,
    pub name: String,
    pub arguments_json: String,
    pub status: ToolCallStatus,
}

pub enum ToolCallStatus {
    Partial,
    Ready,
    Executed,
    Rejected,
}
```

Emission rule:
- Gateway canonical stream emits only `Partial` and `Ready`.
- `Executed` and `Rejected` are reserved for post-gateway tool-execution state managed by Beluna runtime, not emitted by gateway adapters.

## 4) Canonical Event Stream

```rust
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
```

### Event ordering invariants

1. `Started` must be first.
2. At most one terminal event: `Completed` or `Failed`.
3. `Usage` can appear once, before terminal event.
4. `ToolCallReady` may occur multiple times.
5. If stream errors after first emitted data, emit `Failed` and stop (no hidden retry by default).
6. Internal retries are not exposed as extra `Started` events.

## 5) Canonical Final Response (for `infer_once`)

```rust
pub struct CanonicalFinalResponse {
    pub request_id: RequestId,
    pub output_text: String,
    pub tool_calls: Vec<CanonicalToolCall>,
    pub usage: Option<UsageStats>,
    pub finish_reason: FinishReason,
    pub backend_metadata: BTreeMap<String, serde_json::Value>,
}
```

`infer_once` algorithm:

1. Call `infer_stream`.
2. Consume all stream events.
3. Accumulate output text and ready tool calls.
4. Return on `Completed`.
5. Return error on `Failed`.

## 6) Error Taxonomy and Mapping Target

```rust
pub enum GatewayErrorKind {
    InvalidRequest,
    UnsupportedCapability,
    Authentication,
    Authorization,
    RateLimited,
    Timeout,
    CircuitOpen,
    BudgetExceeded,
    BackendTransient,
    BackendPermanent,
    ProtocolViolation,
    Internal,
}

pub struct GatewayError {
    pub kind: GatewayErrorKind,
    pub message: String,
    pub retryable: bool,
    pub backend_id: Option<BackendId>,
    pub provider_code: Option<String>,
    pub provider_http_status: Option<u16>,
}
```

Mapping principles:

- Retry decisions are based on canonical `kind` + `retryable`.
- Raw provider payloads are kept only in redacted debug metadata.

## 7) Usage Model (MVP)

```rust
pub struct UsageStats {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
    pub provider_usage_raw: Option<serde_json::Value>,
}
```

MVP rule:
- usage is best-effort and optional; absence does not fail request.

Status: `READY_FOR_L2_REVIEW`
