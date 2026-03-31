# L1 Plan - Minimal AI Gateway (High-Level Strategy)

- Task Name: `minimal-ai-gateway`
- Stage: `L1` (high-level strategy)
- Date: 2026-02-08
- Status: `DRAFT_FOR_APPROVAL`

## 0) Inputs Locked From L0 Approval

User decisions applied:

1. Backend selection is dialect-driven (local vs remote is irrelevant to core architecture).
2. MVP dialects: `openai_compatible`, `ollama`, `github_copilot_sdk`.
3. Adapter style: raw `reqwest` for HTTP dialects; Copilot via SDK process integration.
4. Cost mode: usage only (no cost estimation in MVP).
5. Streaming API: `Stream<Item = GatewayEvent>`.
6. Reliability policy must be explicit for streaming/tool-calls (safe retry boundaries).

Date-sensitive clarification:

- The Zed post (April 20, 2023) describes an undocumented Copilot LSP integration path.
- As of February 10, 2025, GitHub announced a public Copilot Language Server SDK.
- Strategy: follow the SDK path for MVP (same class of integration pattern: language-server process boundary).

## 1) Strategy Summary

Introduce a dedicated `ai_gateway` module as a strict, typed boundary between Beluna runtime and provider/model backends.

- Callers submit one internal Beluna request.
- RequestNormalizer maps it into a backend-neutral canonical request.
- BackendRouter selects a configured backend profile by ID/dialect and resolves credentials.
- BackendAdapter executes transport + dialect mapping and emits backend-native events.
- ResponseNormalizer converts outputs into a canonical event stream.
- Cross-cutting wrappers (budget policy, reliability, error mapping, telemetry, capabilities) apply uniformly around adapters.

This keeps provider-specific complexity isolated while preserving Beluna's strict protocol design philosophy.

## 2) Target High-Level Architecture

```text
Beluna Runtime
   -> AIGateway (facade)
      -> RequestNormalizer (Beluna request -> canonical request)
      -> BackendRouter (profile selection) + CredentialProvider (secret resolution)
      -> CapabilityGuard (feature checks)
      -> BudgetEnforcer (tokens/time/concurrency/rate smoothing; cost hook)
      -> ReliabilityLayer (timeout + retry/backoff + circuit breaker)
      -> BackendAdapter (transport + dialect mapping)
         - OpenAI-Compatible BackendAdapter (HTTP via reqwest)
         - Ollama BackendAdapter (HTTP via reqwest)
         - GitHub Copilot BackendAdapter (Copilot SDK/LSP over stdio; auth/session lifecycle)
      -> ResponseNormalizer (backend output -> canonical event stream + final usage)
      -> TelemetryEmitter
```

## 3) Core Technical Decisions

1. Internal-first canonical schema
- Create strict internal request/response/event types and a canonical request type.
- RequestNormalizer converts Beluna input into canonical request before any adapter logic.
- BackendAdapters convert canonical request to backend payloads and back.
- Avoid loose pass-through payloads except optional provider metadata fields.

2. BackendAdapter contract (transport + dialect mapping)
- Adapter key is API dialect, not hosting location.
- `openai_compatible` can serve cloud OpenAI-compatible providers or local OpenAI-compatible servers.
- Each BackendAdapter owns both transport details and dialect mapping logic.

3. Unified streaming contract
- Gateway public streaming surface is `Stream<Item = GatewayEvent>`.
- ResponseNormalizer must emit a canonical event stream: text deltas, tool-call deltas, lifecycle markers, terminal usage, and terminal error.

4. Unified tool-call representation
- Normalize all tool calls to one internal structure (`id`, `name`, `arguments_json`, `status`).
- Tool result messages use one internal representation regardless of upstream dialect.

5. Reliability scope (MVP)
- Apply timeout + exponential-backoff retry on transient failures.
- Default retry policy is `before_first_event_only` for streaming requests.
- No retry after first canonical stream event or after a tool-call is emitted unless adapter marks request resumable/idempotent.
- Canonical request carries `request_id` (idempotency key); adapters propagate where supported.
- Include a minimal per-backend circuit breaker (failure-count based) with conservative defaults.
- Retry classifier driven by normalized error kind.

6. Credential management strategy
- Credentials must be resolved via CredentialProvider (or equivalent Router-integrated component), not in adapters/callers.
- BackendAdapter receives resolved auth material via typed context, preventing token/header leakage across layers.

7. Budget enforcement scope (MVP)
- BudgetEnforcer applies per-request and per-backend policies before dispatch and during stream lifecycle.
- MVP enforced budgets: request timeout bound, token/usage ceiling where measurable, per-backend concurrency cap, and rate smoothing.
- Cost ceiling remains a hook only (disabled in MVP due usage-only mode).

8. Usage telemetry scope (MVP)
- Collect latency, attempt count, backend/model, and token usage when provided.
- No cost table and no cost inference in MVP.

9. Copilot SDK strategy
- Integrate via Copilot language server process over stdio.
- Implement as a BackendAdapter that owns both transport and dialect mapping.
- Handle Copilot-specific auth/session lifecycle in adapter-local state.
- Mark unsupported capabilities explicitly (likely no tool-calls/json-mode/vision in MVP).

## 4) Capability Model (High-Level)

Every backend profile exposes declared capabilities:

- `streaming`
- `tool_calls`
- `json_mode`
- `vision`
- `resumable_streaming` (for safe post-first-event retries)

Gateway behavior:

- Validate requested features against backend capabilities before dispatch.
- Return deterministic `UnsupportedCapability` mapped error with backend/dialect context.

## 5) Configuration Strategy

Extend config schema with `ai_gateway` section containing:

- `default_backend`
- `backends[]` profiles:
  - `id`
  - `dialect` (`openai_compatible | ollama | github_copilot_sdk`)
  - connection/auth fields by dialect
  - credential reference (resolved by CredentialProvider)
  - `default_model`
  - optional capability overrides
- reliability defaults:
  - `request_timeout_ms`
  - `max_retries`
  - `backoff_base_ms`
  - `backoff_max_ms`
  - `retry_policy` (`before_first_event_only` default)
  - `breaker_failure_threshold`
  - `breaker_open_ms`
- budget defaults:
  - `max_request_time_ms`
  - `max_usage_tokens_per_request`
  - `max_concurrency_per_backend`
  - `rate_smoothing_per_second`

Config remains schema-validated, aligned with current strict config loading.

## 6) Dependency Requirements (High-Level)

Planned crate additions:

- `reqwest` (HTTP adapters)
- `futures-core` / `futures-util` (typed stream abstractions)
- `tokio-stream` (stream utilities)
- `thiserror` (normalized error taxonomy)
- `tracing` (telemetry events/logging)

No provider SDK crates for HTTP dialects in MVP.

## 7) MVP Endpoint Scope by Dialect

1. `openai_compatible`
- Implement essential text generation + streaming path.
- Support tool-call input/output normalization where dialect-compatible.

2. `ollama`
- Implement essential generation endpoint(s) and streaming.
- Map Ollama-specific response chunks into `GatewayEvent` stream.

3. `github_copilot_sdk`
- Use Copilot language server SDK process transport.
- Implement minimal request path required for text inference streaming, with adapter-owned session/auth lifecycle.
- Treat Copilot adapter as full transport+dialect layer, not a thin mapping wrapper.
- Keep explicit feature limitations surfaced through capability flags.

## 8) Risks and Mitigations

1. Copilot protocol complexity
- Risk: Custom LSP messages and auth flows introduce complexity.
- Mitigation: isolate in dedicated BackendAdapter + process client layer; keep MVP surface minimal.

2. Streaming event mismatch across dialects
- Risk: ordering and partial payload semantics diverge.
- Mitigation: define strict internal event state machine before implementation (L2).

3. Over-scoping reliability features
- Risk: unsafe retries can duplicate streamed/tool side effects.
- Mitigation: enforce retry-before-first-event default and require resumable/idempotent capability for broader retries.

4. Capability drift
- Risk: backend behavior differs from declared capability.
- Mitigation: allow explicit config overrides + conservative defaults.

5. Credential sprawl
- Risk: auth headers/tokens leak across layers and logs.
- Mitigation: force CredentialProvider-mediated resolution and redaction-aware telemetry.

## 9) Deliverables Expected from L2

L2 should specify:

- exact Rust interfaces/traits and module boundaries,
- canonical data structures for request/response/events/errors/canonical-request,
- normalized tool-call structure,
- CredentialProvider interface and secret propagation boundaries,
- BudgetEnforcer policy model and enforcement points,
- retry classifier and backoff algorithm details,
- retry safety semantics for streaming/tool calls (idempotency/resumability),
- dialect-specific mapping tables,
- config structs/schema additions at field level.

## 10) L1 Exit Criteria

L1 is complete when this strategy is approved, specifically:

- dialect-oriented architecture is accepted,
- RequestNormalizer + canonical-request layer is accepted,
- BackendAdapter (transport + dialect) naming and responsibility is accepted,
- CredentialProvider boundary is accepted,
- BudgetEnforcer scope is accepted,
- Copilot SDK integration direction is accepted,
- MVP reliability and telemetry scope is accepted,
- dependency direction is accepted.

Status: `READY_FOR_L2_APPROVAL`
