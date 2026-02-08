# AI Gateway PRD

## Product Statement

AI Gateway is Beluna's provider- and model-agnostic inference boundary.
It standardizes how Beluna runtime sends inference requests and consumes results.

## MVP Scope

- Supported dialects:
  - `openai_compatible` (`chat/completions`-like protocol)
  - `ollama` (`/api/chat`)
  - `github_copilot_sdk` (Copilot language server over stdio JSON-RPC)
- Gateway selects by API dialect, not by local vs remote deployment model.
- Streaming interface: `Stream<Item = GatewayEvent>`.
- Cost model in MVP: usage-only accounting.

## User Stories

- As Beluna runtime, I need one inference boundary so I can call different AI backends through one internal API.
- As Beluna runtime, I need deterministic backend selection so I can avoid hidden fallback behavior.
- As Beluna runtime, I need strict request normalization so invalid tool/message linkage fails early and consistently.
- As Beluna runtime, I need canonical streaming events so upper layers can consume one stable event protocol.
- As Beluna runtime, I need retry/circuit/budget policies so backend failures do not destabilize the system.

## Functional Requirements

- Route each request to exactly one configured backend profile.
- Normalize request/response shapes to Beluna canonical formats.
- Unify tool/function calling semantics into one internal representation.
- Support cross-cutting concerns:
  - auth and endpoint config
  - retries/timeouts/circuit breaker
  - rate limits and transient error mapping
  - telemetry (latency, usage; cost derived from usage when available)
  - capability probing and feature flags

## Non-Goals (MVP)

- No multi-backend fallback.
- No exact protocol parity guarantee for OpenAI-compatible providers.
- No hard dependency on usage availability for in-flight stream termination.

## Flow

1. Runtime submits `BelunaInferenceRequest`.
2. `RequestNormalizer` validates and maps request to `CanonicalRequest`.
3. `BackendRouter` selects one backend profile deterministically.
4. `CredentialProvider` resolves auth material for that backend.
5. `CapabilityGuard` validates requested features against effective backend capabilities.
6. `BudgetEnforcer` applies pre-dispatch checks and acquires backend concurrency/rate budget.
7. `ReliabilityLayer` invokes `BackendAdapter` with retry/backoff and circuit-breaker logic.
8. `BackendAdapter` (transport + dialect mapping) emits backend raw events.
9. `ResponseNormalizer` converts backend raw events to canonical gateway events.
10. Gateway emits canonical stream to caller and propagates cancellation on consumer drop.
11. Budget and telemetry are updated; stream ends with one terminal event.

## Acceptance Criteria

- Backend routing is deterministic and does not perform multi-backend fallback.
- `RequestNormalizer` returns `InvalidRequest` for invalid message/tool linkage states.
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
