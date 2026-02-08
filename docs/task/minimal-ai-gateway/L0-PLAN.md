# L0 Plan - Minimal AI Gateway

- Task Name: `minimal-ai-gateway`
- Stage: `L0` (request + context analysis only)
- Date: 2026-02-08

## 1) Problem Deconstruction

Beluna needs a thin, provider-agnostic inference boundary that:

1. Routes one internal inference request to one configured backend (OpenAI / Anthropic / Gemini / local).
2. Normalizes request and response (including streaming events) into one internal schema.
3. Unifies tool/function calling semantics across providers.
4. Centralizes cross-cutting backend concerns:
   - auth and endpoint config
   - retries, timeouts, circuit breaking
   - rate-limit/transient error mapping
   - telemetry (latency, usage/tokens, estimated cost)
   - capability probing/feature flags (vision, JSON mode, tool calls, streaming)

## 2) Current Codebase Reality (Beluna)

Observed from `/Users/lanzhijiang/Development/Beluna/src`:

- Current runtime is minimal and event-driven:
  - config loading + JSON Schema validation (`config.rs`)
  - Unix socket NDJSON control loop (`server.rs`)
  - strict protocol parsing with `deny_unknown_fields` (`protocol.rs`)
- Protocol style is intentionally strict (aligned with "Avoid Loose protocol design").
- No inference abstraction exists yet.
- No HTTP client/provider SDK dependencies are present in `Cargo.toml`.
- Test surface is currently protocol parsing only (`cargo test` passes 4 tests).

Implication: gateway should be introduced as a new internal module boundary without breaking current socket loop behavior.

## 3) Constraints and Architectural Trade-offs Identified

1. Strictness vs extensibility
- Beluna conventions favor strict schemas and typed messages.
- We should avoid a loose `serde_json::Value` everywhere design; keep an internal typed domain model with explicit conversion layers per provider.

2. Unified model vs provider leakage
- Too-thin normalization leaks provider-specific concepts into call sites.
- Too-aggressive normalization may hide useful backend capability differences.
- Need a stable internal core + optional provider-specific metadata bag.

3. Streaming normalization
- Providers emit different stream event taxonomies (delta text, block events, tool argument chunks, end events).
- Internal stream protocol must preserve ordering, partial tool args, lifecycle terminal states, and error events.

4. Tool/function call unification
- OpenAI, Anthropic, Gemini represent tool calls differently.
- Need one internal tool-call item shape (`id`, `name`, `arguments_json`, `status`) and one tool-result return shape.

5. Resilience behavior surface
- Retry/backoff should apply to transport/transient errors, not deterministic request validation failures.
- Circuit breaking should be per backend target (provider + model/endpoint scope).

6. Capability probing and feature flags
- Some capabilities are static by backend/model family; others are runtime-configurable.
- Need deterministic behavior when caller asks for unsupported features (explicit mapped error).

7. Telemetry and cost
- Tokens/usage availability differs by provider and by success/failure path.
- Cost requires a pricing table source-of-truth (manual config vs built-in table snapshot).

8. Local backend ambiguity
- "local" can mean OpenAI-compatible server, Ollama, llama.cpp, or custom process.
- We need one concrete first-scope definition.

## 4) External Source Findings (for API-shape reality checks)

Primary docs reviewed:

- OpenAI Responses API streaming and function calling:
  - https://platform.openai.com/docs/guides/streaming-responses
  - https://platform.openai.com/docs/guides/function-calling/function-calling
  - https://platform.openai.com/docs/api-reference/responses
- Anthropic Messages API, streaming events, errors/rate limits:
  - https://docs.anthropic.com/en/api/messages
  - https://docs.anthropic.com/en/api/messages-streaming
  - https://docs.anthropic.com/en/api/errors
  - https://docs.anthropic.com/en/api/rate-limits
- Gemini API function calling and API overview:
  - https://ai.google.dev/gemini-api/docs/function-calling
  - https://ai.google.dev/api

Key implications for Beluna gateway design:

- All three support streaming but with different event vocabularies, so normalization cannot be a 1:1 pass-through.
- Tool calling exists across all three, but call/result payload shapes differ and must be canonicalized.
- Rate limit and overload signaling differ (e.g., provider-specific headers/status details), requiring a normalized error taxonomy with raw-provider details preserved.

## 5) Initial Scope Boundary (L0 Recommendation)

Recommend explicitly scoping MVP to:

- Single-turn inference call API (request -> response) with optional streaming callback/channel.
- Provider adapters for OpenAI, Anthropic, Gemini, and a defined local backend type.
- No conversation memory orchestration in gateway itself.
- No provider SDK lock-in requirement yet (can use raw HTTP + serde for determinism).

## 6) Open Questions Requiring User Decision

1. Local backend definition
- Choose one for MVP:
  - OpenAI-compatible HTTP endpoint
  - Ollama native API
  - custom Beluna local process protocol

2. Primary integration style
- Prefer raw HTTP adapters in Rust (`reqwest`) or provider SDK crates where available?

3. Cost accounting mode for MVP
- Option A: usage/tokens only (no cost)
- Option B: usage + configurable static pricing table in config

4. Reliability defaults
- Retry max attempts / timeout defaults / circuit breaker policy target for MVP.

5. Streaming API to callers
- Should internal API expose:
  - async stream (`Stream<Item = GatewayEvent>`)
  - callback/event channel abstraction

## 7) Proposed Working Assumptions (if you do not override)

- Local backend = OpenAI-compatible HTTP endpoint.
- Use raw HTTP (`reqwest`) for all adapters for consistent behavior.
- Cost = optional best-effort using configurable per-model pricing in config.
- Streaming = typed internal async event stream.
- Circuit breaker = simple per-backend rolling-failure breaker (MVP).

## 8) Exit Criteria for L0

This stage is complete when:

- request is deconstructed,
- existing architecture constraints are documented,
- trade-offs are identified,
- external API behavior differences are validated,
- open decisions are made explicit for gating.

Status: `READY_FOR_L1_APPROVAL`
