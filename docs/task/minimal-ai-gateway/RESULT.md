# RESULT - Minimal AI Gateway

- Task Name: `minimal-ai-gateway`
- Date: 2026-02-08
- Status: `IMPLEMENTED`

## 1) Objective and Delivered Scope

Implemented a minimal AI Gateway module for Beluna as a provider/model-agnostic inference boundary with:

- dialect-based backend routing (`openai_compatible`, `ollama`, `github_copilot_sdk`)
- strict request normalization into canonical request types
- canonical event stream output (`Started`, deltas/tool events, `Usage`, terminal)
- unified tool-call representation
- reliability controls (retry/backoff + minimal circuit breaker)
- budget controls (timeout, per-backend concurrency, rate smoothing, usage post-check accounting)
- credential resolution boundary and telemetry hooks

## 2) Implemented Architecture Snapshot

Implemented module tree:

- `src/ai_gateway/mod.rs`
- `src/ai_gateway/types.rs`
- `src/ai_gateway/error.rs`
- `src/ai_gateway/gateway.rs`
- `src/ai_gateway/router.rs`
- `src/ai_gateway/request_normalizer.rs`
- `src/ai_gateway/response_normalizer.rs`
- `src/ai_gateway/capabilities.rs`
- `src/ai_gateway/credentials.rs`
- `src/ai_gateway/budget.rs`
- `src/ai_gateway/reliability.rs`
- `src/ai_gateway/telemetry.rs`
- `src/ai_gateway/adapters/mod.rs`
- `src/ai_gateway/adapters/http_common.rs`
- `src/ai_gateway/adapters/openai_compatible.rs`
- `src/ai_gateway/adapters/ollama.rs`
- `src/ai_gateway/adapters/copilot_rpc.rs`
- `src/ai_gateway/adapters/github_copilot.rs`

## 3) Config and Schema Changes

Updated config loading and schema to include strict `ai_gateway` configuration:

- `src/config.rs`
- `beluna.schema.json`
- `beluna.jsonc`

Key config additions:

- `ai_gateway.default_backend`
- `ai_gateway.backends[]` with `dialect`, `endpoint`, `credential`, `default_model`
- tagged `credential` object shape (`env` / `inline_token` / `none`)
- reliability and budget sections

## 4) Reliability and Cancellation Behavior

Implemented behavior:

- retry with exponential backoff
- retry safety guard:
  - default policy retries only before first output/tool event
  - no retry after output/tool events unless policy/capability allow it
- per-backend minimal circuit breaker
- deterministic cancellation propagation:
  - when stream consumer drops, adapter cancel handle is invoked
  - budget/concurrency lease is released
  - cancellation does not count as breaker failure

## 5) Adapter Support Status

### OpenAI-compatible

- implemented via HTTP (`chat/completions`-like protocol)
- SSE parsing for stream mode
- graceful handling of protocol divergence where possible

### Ollama

- implemented via HTTP (`/api/chat`)
- NDJSON stream parsing
- terminal usage mapping when present

### Copilot SDK/LSP

- implemented via stdio JSON-RPC process transport
- initialize + auth readiness check path (`checkStatus`)
- completion path via `textDocument/copilotPanelCompletion` with fallback to `textDocument/inlineCompletion`

## 6) Tests Executed and Results

Commands run:

- `cargo fmt`
- `cargo test`

Result:

- `22` tests passed
- `0` failed

Test coverage includes:

- request normalization invariants
- deterministic routing/no fallback
- reliability retry boundaries and breaker behavior
- budget precheck + concurrency enforcement
- adapter basic validation paths
- gateway e2e retry/no-retry behavior

## 7) Deviations from L3 Plan

1. Copilot adapter integration uses a lightweight internal JSON-RPC transport implementation rather than an external SDK crate; behavior is aligned with the planned SDK/LSP transport direction but remains conservative.
2. Usage token budget enforcement is implemented as best-effort accounting (as planned) and does not terminate in-flight streams.

## 8) Remaining Limitations and Next Steps

- AI Gateway is implemented as module boundary and config/schema capability; it is not yet exposed through the existing Unix socket runtime protocol.
- Copilot method/response shape assumptions are conservative and may need adjustment against exact SDK/server versions.
- No live provider-network integration tests are run in CI (mock/in-process tests only).

## 9) Invariant Confirmation

Confirmed in implementation and tests:

- deterministic routing, no multi-backend fallback
- strict tool-message linkage validation in `RequestNormalizer`
- gateway stream does not emit `ToolCallStatus::Executed` or `ToolCallStatus::Rejected`
- cancellation-on-drop invokes adapter cancellation and releases budget resources

## 10) Product Docs Updated

Updated product documents to include explicit AI Gateway feature documentation and glossary terms:

- `docs/product/overview.md`
- `docs/product/README.md`
- `docs/product/glossary.md`
- `docs/features/ai-gateway/README.md`
- `docs/features/ai-gateway/PRD.md`
- `docs/features/ai-gateway/HLD.md`
- `docs/features/ai-gateway/LLD.md`
