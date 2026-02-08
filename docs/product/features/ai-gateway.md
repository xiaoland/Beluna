# AI Gateway

## User Stories

- As Beluna runtime, I need one inference boundary so I can call different AI backends through one internal API.
- As Beluna runtime, I need deterministic backend selection so I can avoid hidden fallback behavior.
- As Beluna runtime, I need strict request normalization so invalid tool/message linkage fails early and consistently.
- As Beluna runtime, I need canonical streaming events so upper layers can consume one stable event protocol.
- As Beluna runtime, I need retry/circuit/budget policies so backend failures do not destabilize the system.
- As Beluna runtime, I need support for `openai_compatible`, `ollama`, and `github_copilot_sdk` in MVP.

## Flow

1. Runtime submits `BelunaInferenceRequest`.
2. RequestNormalizer validates and maps request to `CanonicalRequest`.
3. BackendRouter selects one backend profile deterministically.
4. CredentialProvider resolves auth material for that backend.
5. CapabilityGuard validates requested features against effective backend capabilities.
6. BudgetEnforcer applies pre-dispatch checks and acquires backend concurrency/rate budget.
7. ReliabilityLayer invokes BackendAdapter with retry/backoff and circuit-breaker logic.
8. BackendAdapter (transport + dialect mapping) emits backend raw events.
9. ResponseNormalizer converts backend raw events to canonical gateway events.
10. Gateway emits canonical stream to caller and propagates cancellation on consumer drop.
11. Budget and telemetry are updated; stream ends with one terminal event.

## Acceptance Criteria

- Backend routing is deterministic and does not perform multi-backend fallback.
- RequestNormalizer returns `InvalidRequest` for invalid message/tool linkage states.
- Gateway canonical stream starts with `Started` and ends with exactly one terminal event (`Completed` or `Failed`).
- Default retry policy retries only before first output/tool event.
- Circuit breaker can open per backend after repeated transient failures.
- Budget enforces timeout, per-backend concurrency, and rate smoothing.
- Usage token post-check is best-effort accounting and does not terminate active streams.
- Adapters are available for:
  - `openai_compatible` (`chat/completions`-like)
  - `ollama` (`/api/chat`)
  - `github_copilot_sdk` (Copilot language server over stdio JSON-RPC)

## Glossary

- Backend Dialect: API protocol family used by a backend profile (MVP: `openai_compatible`, `ollama`, `github_copilot_sdk`).
- Backend Adapter: Component that owns both transport and dialect mapping for one backend dialect.
- Canonical Request: Backend-neutral internal inference request shape after `RequestNormalizer`.
- Canonical Event Stream: Backend-neutral streaming output protocol used by Beluna internals.
- RequestNormalizer: Validation and mapping layer that rejects invalid input states before backend dispatch.
- BackendRouter: Deterministic backend selector (no multi-backend fallback in MVP).
- CapabilityGuard: Validator for requested features (tool calls, vision, JSON mode, streaming) against backend capability flags.
- BudgetEnforcer: Layer that applies timeout, concurrency, and rate-smoothing policy; usage token post-check is best-effort accounting.
- ReliabilityLayer: Retry/backoff and circuit-breaker layer with safe retry boundaries for streaming/tool semantics.
- CredentialProvider: Secret resolution boundary used by gateway adapters.
- Stream Drop Cancellation: Behavior where dropping consumer stream cancels in-flight backend work and releases resources.

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
