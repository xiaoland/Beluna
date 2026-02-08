# L2-03 - Reliability, Budget, Credentials, Telemetry

- Task Name: `minimal-ai-gateway`
- Stage: `L2` detail: policies and algorithms
- Date: 2026-02-08
- Status: `DRAFT_FOR_APPROVAL`

## 1) Reliability Policy (Streaming + Tool Calls)

Default policy: `before_first_event_only`.

Meaning:

1. Retry is allowed only if no canonical output event has been emitted.
2. Retry is disabled after first canonical stream event.
3. Retry is disabled after any `ToolCallDelta` or `ToolCallReady` unless backend declares resumable+idempotent behavior.

Optional policy (future): `adapter_resumable`.

## 2) Retry Algorithm (Exponential Backoff)

Parameters:

- `max_retries`
- `backoff_base_ms`
- `backoff_max_ms`
- jitter ratio: fixed 20%

Pseudo-code:

```text
attempt = 0
loop:
  breaker.allow_or_error()
  start_time = now
  result = adapter.invoke_stream(...)

  if result is immediate transport/setup error:
    classify and maybe_retry(before_first_event=true)

  stream = result.stream
  emitted_output = false
  emitted_tool = false

  while event = stream.next():
    if consumer_dropped_stream:
      if result.cancel exists: call result.cancel
      release_budget_resources()
      return cancelled

    normalized = response_normalizer.map(event)
    if normalized is output_text_delta or tool_call_delta or tool_call_ready:
      emitted_output = true
    if normalized is tool_call_delta or tool_call_ready:
      emitted_tool = true

    forward normalized

  if terminal success:
    breaker.record_success()
    return

  if terminal error:
    can_retry = classifier.retryable(error)
      && attempt < max_retries
      && (!emitted_output || retry_policy_allows_post_start)
      && (!emitted_tool || adapter.is_tool_retry_safe())

    if can_retry:
      breaker.record_failure_transient()
      sleep(backoff_with_jitter(attempt))
      attempt += 1
      continue

    breaker.record_failure_terminal()
    return error
```

Consumer-drop cancellation semantics:

1. The canonical stream wrapper owns the adapter cancel handle.
2. On stream drop, wrapper invokes cancel handle and releases acquired budget/concurrency resources.
3. Consumer-initiated cancellation is treated as cancellation, not backend failure, for retry/breaker accounting.

## 3) Circuit Breaker (Minimal MVP)

Per backend state:

```rust
pub struct BreakerState {
    failure_streak: u32,
    open_until: Option<Instant>,
}
```

Behavior:

1. Closed: allow requests.
2. Open: reject with `GatewayErrorKind::CircuitOpen` until `open_until`.
3. After `open_until`: allow one probe request.
4. Probe success -> reset breaker.
5. Probe failure -> reopen breaker.

Failure counting:
- Count transient backend failures and timeouts.
- Do not count caller-side invalid request failures.

## 4) BudgetEnforcer (MVP)

### Enforced budgets

1. `max_request_time_ms`
- Effective timeout = min(global default, request override, backend override).

2. `max_usage_tokens_per_request`
- Pre-check against requested max output token hints where available.
- Post-check against reported usage is best-effort only.
- If usage is missing or only available at terminal event, no mid-stream enforcement is attempted.
- Exceeded post-check updates budget accounting/telemetry and may influence future request admission; it does not terminate an already-running stream.

3. `max_concurrency_per_backend`
- Use per-backend `Semaphore`.
- Acquire permit before adapter invocation, release on stream terminal.

4. `rate_smoothing_per_second`
- Simple token bucket (request-level) per backend.
- Delay request start until budget token available.

## 5) Credential Provider Boundary

### Credential reference model

```rust
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CredentialRef {
    Env { var: String },
    InlineToken { token: String },
    None,
}

pub struct ResolvedCredential {
    pub auth_header: Option<String>,
    pub extra_headers: Vec<(String, String)>,
    pub opaque: BTreeMap<String, String>,
}
```

Rules:

1. `BackendProfile` stores only credential references, never resolved tokens.
2. Only `CredentialProvider` resolves secrets.
3. Adapters receive resolved credential via `AdapterContext`.
4. Telemetry/logging must redact all resolved secret values.

## 6) Telemetry Event Schema

```rust
pub enum GatewayTelemetryEvent {
    RequestStarted {
        request_id: RequestId,
        backend_id: BackendId,
        model: String,
    },
    AttemptStarted {
        request_id: RequestId,
        attempt: u32,
    },
    AttemptFailed {
        request_id: RequestId,
        attempt: u32,
        kind: GatewayErrorKind,
        retryable: bool,
    },
    StreamFirstEvent {
        request_id: RequestId,
        latency_ms: u64,
    },
    RequestCompleted {
        request_id: RequestId,
        attempts: u32,
        total_latency_ms: u64,
        usage: Option<UsageStats>,
    },
    RequestFailed {
        request_id: RequestId,
        attempts: u32,
        total_latency_ms: u64,
        error_kind: GatewayErrorKind,
    },
}
```

## 7) Reliability + Budget Interaction Rules

1. If timeout occurs before first event, retry may apply.
2. If timeout occurs after first event, no retry by default.
3. If BudgetEnforcer rejects request pre-dispatch, do not touch breaker state.
4. Post-check token budget exceed is accounting-only in MVP and does not terminate an in-flight stream.
5. If stream is cancelled by consumer drop, do not count it as backend failure for circuit breaker state.

## 8) Idempotency + Request ID Rules

1. `CanonicalRequest.request_id` is required.
2. HTTP adapters send `X-Request-Id` and provider-specific idempotency headers where supported.
3. Copilot adapter includes request ID in JSON-RPC metadata.

Status: `READY_FOR_L2_REVIEW`
