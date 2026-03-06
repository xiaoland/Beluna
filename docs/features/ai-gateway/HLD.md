# AI Gateway HLD

## High-Level Architecture

AI Gateway is a thin boundary between Beluna runtime and backend dialects.
It is backend-dialect oriented and backend transport agnostic.

## Component Model

- `Chat`: public facade (`open_thread`, `clone_thread_with_turns`, `query_turns`)
- `Thread`: backend-bound conversation aggregate (`complete`)
- `Turn`: atomic unit with ordered `Message` array
- `Message`: explicit concrete message layer
- `BackendRouter`: deterministic backend profile selection
- `CredentialProvider`: secret/token resolution boundary
- `CapabilityGuard`: feature validation against backend capabilities
- `ResilienceEngine`: retry/backoff/circuit + timeout + concurrency/rate admission
- `BackendAdapter`: owns both transport and dialect mapping
- `ToolScheduler`: executes tool call and appends linked result message atomically
- `tracing` structured telemetry: lifecycle/attempt/outcome/usage events emitted as logs

## Routing and Adapter Principles

- Router selection is deterministic; exactly one backend is selected.
- MVP has no multi-backend fallback.
- `BackendAdapter` is transport + dialect together.
- Copilot adapter is also transport + dialect (SDK/LSP lifecycle included), not HTTP-only mapping.
- OpenAI-compatible means `chat/completions`-like compatibility; exact parity is not assumed and missing fields degrade gracefully.

## Resilience Semantics

- Retry policy uses exponential backoff.
- Streaming retry is allowed only before first output/tool event by default.
- No hidden retry after output/tool events unless resumable semantics are explicitly implemented.
- Circuit breaker counts transient backend failures.
- Caller-side cancellation (consumer drops stream) is not counted as backend failure.

## Usage Semantics

- Resilience enforces timeout bound, per-backend concurrency limit, and rate smoothing.
- Gateway budget rejection is removed.
- Usage remains best-effort output metadata for caller-owned policy.

## Security and Credentials

- All secrets are resolved through `CredentialProvider`.
- Do not leak tokens/headers in errors, logs, or telemetry payloads.
- Keep redaction defaults for provider/raw metadata.

## Key Runtime Invariants

- Canonical stream starts with `Started`.
- Canonical stream ends with exactly one terminal event (`Completed` or `Failed`).
- Gateway never emits `ToolCallStatus::Executed` or `ToolCallStatus::Rejected`.
- Stream drop cancels in-flight backend work and releases resilience lease resources.

## Involved Surfaces

- `src/ai_gateway/chat/api_chat.rs`
- `src/ai_gateway/chat/thread.rs`
- `src/ai_gateway/chat/turn.rs`
- `src/ai_gateway/chat/message.rs`
- `src/ai_gateway/chat/runtime.rs`
- `src/ai_gateway/chat/tool_scheduler.rs`
- `src/ai_gateway/router.rs`
- `src/ai_gateway/capabilities.rs`
- `src/ai_gateway/resilience.rs`
- `src/ai_gateway/credentials.rs`
- `src/ai_gateway/adapters/openai_compatible/chat.rs`
- `src/ai_gateway/adapters/ollama/chat.rs`
- `src/ai_gateway/adapters/github_copilot/chat.rs`
- `src/ai_gateway/types.rs`
- `src/ai_gateway/error.rs`
- `src/ai_gateway/telemetry.rs`
- `src/config.rs`
- `beluna.schema.json`
- `beluna.jsonc`

## Related Module Docs

- `docs/modules/ai-gateway/purpose.md`
- `docs/modules/ai-gateway/architecture.md`
- `docs/modules/ai-gateway/execution-flow.md`
- `docs/modules/ai-gateway/adapters.md`
- `docs/modules/ai-gateway/configuration.md`
- `docs/modules/ai-gateway/policies.md`
