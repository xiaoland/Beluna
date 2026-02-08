# AI Gateway HLD

## High-Level Architecture

AI Gateway is a thin boundary between Beluna runtime and backend dialects.
It is backend-dialect oriented and backend transport agnostic.

## Component Model

- `AIGateway`: facade (`infer_stream`, `infer_once`)
- `RequestNormalizer`: internal request validation + canonical mapping
- `BackendRouter`: deterministic backend profile selection
- `CredentialProvider`: secret/token resolution boundary
- `CapabilityGuard`: feature validation against backend capabilities
- `BudgetEnforcer`: timeout, concurrency, and rate smoothing
- `ReliabilityLayer`: exponential backoff retry + circuit breaker
- `BackendAdapter`: owns both transport and dialect mapping
- `ResponseNormalizer`: backend raw events -> canonical event stream
- `TelemetrySink`: lifecycle/attempt/outcome/usage telemetry

## Routing and Adapter Principles

- Router selection is deterministic; exactly one backend is selected.
- MVP has no multi-backend fallback.
- `BackendAdapter` is transport + dialect together.
- Copilot adapter is also transport + dialect (SDK/LSP lifecycle included), not HTTP-only mapping.
- OpenAI-compatible means `chat/completions`-like compatibility; exact parity is not assumed and missing fields degrade gracefully.

## Reliability Semantics

- Retry policy uses exponential backoff.
- Streaming retry is allowed only before first output/tool event by default.
- No hidden retry after output/tool events unless resumable semantics are explicitly implemented.
- Circuit breaker counts transient backend failures.
- Caller-side cancellation (consumer drops stream) is not counted as backend failure.

## Budget and Usage Semantics

- Enforce timeout bound, per-backend concurrency limit, and rate smoothing.
- Usage post-check is best-effort:
  - usage may be missing
  - usage may arrive late
  - post-check must not terminate an active stream
  - post-check may affect future-request accounting/policy only

## Security and Credentials

- All secrets are resolved through `CredentialProvider`.
- Do not leak tokens/headers in errors, logs, or telemetry payloads.
- Keep redaction defaults for provider/raw metadata.

## Key Runtime Invariants

- Canonical stream starts with `Started`.
- Canonical stream ends with exactly one terminal event (`Completed` or `Failed`).
- Gateway never emits `ToolCallStatus::Executed` or `ToolCallStatus::Rejected`.
- Stream drop cancels in-flight backend work and releases budget/concurrency resources.

## Involved Surfaces

- `src/ai_gateway/gateway.rs`
- `src/ai_gateway/request_normalizer.rs`
- `src/ai_gateway/router.rs`
- `src/ai_gateway/capabilities.rs`
- `src/ai_gateway/budget.rs`
- `src/ai_gateway/reliability.rs`
- `src/ai_gateway/response_normalizer.rs`
- `src/ai_gateway/credentials.rs`
- `src/ai_gateway/adapters/openai_compatible.rs`
- `src/ai_gateway/adapters/ollama.rs`
- `src/ai_gateway/adapters/github_copilot.rs`
- `src/ai_gateway/adapters/copilot_rpc.rs`
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
