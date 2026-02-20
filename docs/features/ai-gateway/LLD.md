# AI Gateway LLD

## Canonical Request Rules

RequestNormalizer must enforce deterministic validation before adapter dispatch.

- If `role == Tool`:
  - `tool_call_id` must exist
  - `tool_name` should exist (required for dialects that require it)
  - `parts` should be text/json style payloads; image URL parts are invalid
- If `role != Tool`:
  - `tool_call_id` must be `None`
  - `tool_name` must be `None`

Invalid states must fail as canonical `InvalidRequest`, not provider-specific errors.

## Canonical Event Stream

- Stream type: `Stream<Item = GatewayEvent>`.
- First event: `Started`.
- Exactly one terminal event: `Completed` or `Failed`.
- Gateway never emits:
  - `ToolCallStatus::Executed`
  - `ToolCallStatus::Rejected`

## Retry, Cancellation, and Side-Effect Safety

- Default retry with exponential backoff applies only before first output/tool event.
- After output/tool events, retry is disabled unless explicit resumable/idempotent behavior exists.
- Consumer drop cancels underlying backend request/session and releases acquired resources.

## Usage and Budget Detail

- Budget pre-checks enforce timeout/concurrency/rate limits before dispatch.
- Usage-token post-check is best-effort:
  - optional usage cannot be assumed
  - late usage cannot retroactively stop current stream
  - post-check may only update future accounting/policy signals

## Adapter-Level Constraints

- Adapter boundary combines transport + dialect mapping (`BackendAdapter`).
- OpenAI-compatible adapter targets chat-completions-like behavior, not strict parity.
- Missing provider fields should degrade gracefully where safe.

## Contracts and Test Mapping

- Contracts:
  - `docs/contracts/ai-gateway/request-normalizer.md`
  - `docs/contracts/ai-gateway/router.md`
  - `docs/contracts/ai-gateway/reliability.md`
  - `docs/contracts/ai-gateway/budget.md`
  - `docs/contracts/ai-gateway/adapters.md`
  - `docs/contracts/ai-gateway/gateway-stream.md`
- Tests:
  - `tests/ai_gateway/*`

## Implementation References

- Core: `src/ai_gateway/gateway.rs`
- Normalization: `src/ai_gateway/request_normalizer.rs`, `src/ai_gateway/response_normalizer.rs`
- Routing/Capabilities: `src/ai_gateway/router.rs`, `src/ai_gateway/capabilities.rs`
- Reliability/Budget: `src/ai_gateway/reliability.rs`, `src/ai_gateway/budget.rs`
- Credentials/Tracing Telemetry: `src/ai_gateway/credentials.rs`, `src/ai_gateway/telemetry.rs`
- Adapters: `src/ai_gateway/adapters/*`
