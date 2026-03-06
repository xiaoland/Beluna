# AI Gateway LLD

## Turn And Message Invariants

`Turn` and `Thread` enforce deterministic validation before adapter dispatch.

- `Turn` is the atomic thread-history unit and owns an ordered `Vec<Message>`.
- `ToolCallMessage` must be immediately followed by a matching `ToolCallResultMessage`.
- `ToolCallResultMessage` cannot appear without a preceding matching `ToolCallMessage`.
- `Turn.append_one(ToolCallMessage)` must schedule the tool and append both call and result in one logical operation.
- `Turn.truncate_one()` must remove a whole tool-call bundle when the tail is a tool result.

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

## Usage and Resilience Detail

- Resilience pre-dispatch enforces timeout/concurrency/rate limits before dispatch.
- Usage reporting is best-effort:
  - optional usage cannot be assumed
  - late usage cannot retroactively stop current stream
  - caller owns follow-up budget/accounting policy

## Adapter-Level Constraints

- Adapter boundary combines transport + dialect mapping (`BackendAdapter`).
- OpenAI-compatible adapter targets chat-completions-like behavior, not strict parity.
- OpenAI-compatible message `content` must stay wire-safe (`string` or `array<object>`); JSON parts are stringified at transport mapping.
- Missing provider fields should degrade gracefully where safe.

## Contracts and Test Mapping

- Contracts:
- `docs/contracts/ai-gateway/chat-invariants.md`
- `docs/contracts/ai-gateway/router.md`
- `docs/contracts/ai-gateway/resilience.md`
- `docs/contracts/ai-gateway/usage.md`
  - `docs/contracts/ai-gateway/adapters.md`
  - `docs/contracts/ai-gateway/gateway-stream.md`
- Tests:
  - `tests/ai_gateway/*`

## Implementation References

- Core: `src/ai_gateway/chat/api_chat.rs`, `src/ai_gateway/chat/thread.rs`, `src/ai_gateway/chat/runtime.rs`
- Message/Turn model: `src/ai_gateway/chat/message.rs`, `src/ai_gateway/chat/turn.rs`
- Routing/Capabilities: `src/ai_gateway/router.rs`, `src/ai_gateway/chat/capabilities.rs`
- Resilience: `src/ai_gateway/resilience.rs`
- Credentials/Tracing Telemetry: `src/ai_gateway/credentials.rs`, `src/ai_gateway/telemetry.rs`
- Adapters: `src/ai_gateway/adapters/*`
