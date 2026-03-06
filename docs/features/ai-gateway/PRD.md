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
- As Beluna runtime, I need strict turn/message invariants so invalid tool linkage fails early and consistently.
- As Beluna runtime, I need canonical streaming events so upper layers can consume one stable event protocol.
- As Beluna runtime, I need retry/circuit/concurrency/rate policies so backend failures do not destabilize the system.

## Functional Requirements

- Route each request to exactly one configured backend profile.
- Normalize thread history and turn input into one backend dispatch payload.
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

1. Runtime opens or reuses a backend-bound `Thread`.
2. `Thread` validates message/turn invariants and builds `TurnPayload` from prior turns plus current input.
3. `BackendRouter` selects one backend profile deterministically when the thread is opened or cloned.
4. `CredentialProvider` resolves auth material for that backend.
5. `CapabilityGuard` validates requested features against effective backend capabilities.
6. `ResilienceEngine` applies timeout/concurrency/rate admission and retry/circuit controls.
7. `BackendAdapter` (transport + dialect mapping) emits backend raw events.
8. `Thread` materializes one new `Turn`, appending tool-call/result bundles atomically when tools are invoked.
9. Telemetry and usage metadata are updated; stream ends with one terminal event.

## Acceptance Criteria

- Backend routing is deterministic and does not perform multi-backend fallback.
- Invalid tool-call/result linkage fails as `InvalidRequest` before backend dispatch.
- Gateway canonical stream starts with `Started` and ends with exactly one terminal event (`Completed` or `Failed`).
- Default retry policy retries only before first output/tool event.
- Circuit breaker can open per backend after repeated transient failures.
- Resilience enforces timeout, per-backend concurrency, and rate smoothing.
- Gateway does not enforce token budget rejection; usage is returned for caller policy.
- Adapters are available for:
  - `openai_compatible` (`chat/completions`-like)
  - `ollama` (`/api/chat`)
  - `github_copilot_sdk` (Copilot language server over stdio JSON-RPC)

## Glossary

- Backend Dialect: API protocol family used by a backend profile (MVP: `openai_compatible`, `ollama`, `github_copilot_sdk`).
- Backend Adapter: Component that owns both transport and dialect mapping for one backend dialect.
- TurnPayload: Backend-neutral internal dispatch payload built from thread history plus current turn input.
- Canonical Event Stream: Backend-neutral streaming output protocol used by Beluna internals.
- Turn Invariant Validation: Validation layer enforced by `Turn` and `Thread` before backend dispatch.
- BackendRouter: Deterministic backend selector (no multi-backend fallback in MVP).
- CapabilityGuard: Validator for requested features (tool calls, vision, JSON mode, streaming) against backend capability flags.
- ResilienceEngine: Layer that applies retry/backoff/circuit policy with timeout and per-backend concurrency/rate controls.
- CredentialProvider: Secret resolution boundary used by gateway adapters.
- Stream Drop Cancellation: Behavior where dropping consumer stream cancels in-flight backend work and releases resources.
