# AI Gateway OpenAI Responses Backend

## MVT Core

- Objective & Hypothesis: Add an explicit OpenAI Responses backend adapter for AI Gateway Chat so Cortex can use OpenAI's Responses API through the existing `Thread::complete(...)` path with tool-call-native Primary turns, structured helper output, deterministic routing, and canonical Beluna-owned thread history.
- Guardrails Touched: AI Gateway adapter code owns provider wire translation only; Chat runtime keeps route resolution, capability checks, resilience, retry, thread commits, and observability ownership. Cortex keeps cognition orchestration and must continue to depend on AI Gateway Chat contracts rather than provider-native state.
- Verification: Rust tests prove request body mapping, response parsing, tool-call continuation mapping, structured output mapping, route selection for the new dialect, and error classification. `cargo test -p beluna-core ai_gateway` or the closest focused test target must pass before code closure.

## Exploration Scaffold

- Perturbation: Cortex currently needs a backend adapter that speaks OpenAI Responses API rather than only Chat Completions-style OpenAI-compatible wire format.
- Input Type: Intent.
- Active Mode or Transition Note: Execute began after human confirmation on 2026-05-02.
- Governing Anchors:
  - `AGENTS.md`
  - `tasks/README.md`
  - `core/AGENTS.md`
  - `core/src/ai_gateway/AGENTS.md`
  - `core/src/cortex/AGENTS.md`
  - `docs/30-unit-tdd/core/design.md`
  - `docs/30-unit-tdd/core/interfaces.md`
  - `docs/20-product-tdd/cross-unit-contracts.md`
  - `ai-gateway/TOPOLOGY.md`
- Impact Hypothesis:
  - The change should be localized to AI Gateway config/adapter registration and new adapter wire mapping.
  - Cortex should need no production code changes if the adapter returns the existing `BackendCompleteResponse` shape correctly.
  - Tests should exercise provider wire semantics without live network calls.
- Temporary Assumptions:
  - The new dialect should be named `openai_responses` in config and represented by a new `BackendDialect` variant.
  - The adapter should use `POST /responses` under the configured endpoint.
  - The adapter should send `store: false` and replay Beluna Chat thread messages as `input[]`.
  - The adapter should implement `complete()` first; `stream()` can return `UnsupportedCapability` for this task because Cortex drives `Thread::complete(...)`.
  - Responses API function tools can be represented from existing `ChatToolDefinition` without adding a new Beluna tool schema.
  - Responses API `function_call` output items map to existing `ToolCallResult` with `ToolCallStatus::Ready`.
  - Existing `BackendCapabilities` fields are sufficient for the first slice: `tool_calls`, `parallel_tool_calls`, `json_mode`, `json_schema_mode`, and `streaming`.
- Negotiation Triggers:
  - OpenAI Responses API requires provider-managed conversation state for a Cortex path.
  - Existing `ChatMessage` cannot encode a required Responses input item without changing the Chat contract.
  - Existing `BackendCapabilities` cannot describe a necessary runtime decision.
  - Tool-call continuation requires changing `Thread::complete(...)` commit semantics.
  - Implementation would cross from adapter-local translation into Cortex orchestration or durable product contract changes.
- Promotion Candidates:
  - If the adapter becomes the preferred OpenAI path, promote config examples and capability notes into Core Unit TDD.
  - If `store: false` plus full replay becomes a standing provider-state rule, promote it into AI Gateway design memory.
  - If Responses streaming becomes required, open a separate task because it touches runtime stream semantics and observability.

## Current Findings

- Existing adapter seam: `core/src/ai_gateway/adapters/mod.rs` defines `BackendAdapter` with `complete(...)`, `stream(...)`, `static_capabilities()`, and `supports_tool_retry()`.
- Existing route seam: `core/src/ai_gateway/types.rs` defines `BackendDialect`; `core/src/ai_gateway/router.rs` maps configured aliases to backend/model targets.
- Existing complete flow: `Thread::complete(...)` builds a `TurnPayload`, `ChatRuntime::dispatch_complete(...)` performs capability/resilience handling, then calls the selected adapter's `complete(...)`.
- Cortex path: `core/src/cortex/runtime/primary.rs` uses `Thread::complete(...)` for Primary, Attention, Cleanup, and helper organs. Primary uses dynamic tools and expects tool calls to become same-tick continuations.
- Canonical history: `core/src/ai_gateway/chat/thread.rs` commits turns and tool results in Beluna memory; provider-native conversation state is an execution medium only.
- OpenAI Responses API facts from official docs:
  - Responses creation supports `input`, `instructions`, `tools`, `tool_choice`, `parallel_tool_calls`, `text`, `store`, and `max_output_tokens`.
  - Function calls are returned as response output items with `type: "function_call"`, `call_id`, `name`, and `arguments`.
  - Function call results are sent back as input items with `type: "function_call_output"`, `call_id`, and `output`.
  - Structured output for Responses is expressed through `text.format`.

## Proposed File Scope

- Modify `core/src/ai_gateway/types.rs` to add the new dialect.
- Modify `core/src/ai_gateway/adapters/mod.rs` to register the new adapter.
- Add `core/src/ai_gateway/adapters/openai_responses/mod.rs`.
- Add `core/src/ai_gateway/adapters/openai_responses/chat.rs`.
- Add `core/src/ai_gateway/adapters/openai_responses/wire.rs`.
- Add focused tests under `core/tests/ai_gateway/` or the nearest existing adapter test location.

## Minimal Backend Behavior

- Request mapping:
  - System messages become `instructions` when possible, or equivalent input items if multiple system messages appear.
  - User messages become `message` input items with user role.
  - Assistant text messages become assistant message input items.
  - Assistant tool-call messages become `function_call` input items.
  - Tool result messages become `function_call_output` input items.
  - Tools become Responses function tools with `name`, `description`, `parameters`, and strict schema where supported by the existing schema.
  - `OutputMode::Text` maps to default text output.
  - `OutputMode::JsonObject` maps to Responses text JSON object format.
  - `OutputMode::JsonSchema` maps to Responses text JSON schema format.
- Response mapping:
  - Text output items aggregate into `BackendCompleteResponse.output_text`.
  - Function call output items become `ToolCallResult { status: Ready }`.
  - Usage maps `input_tokens`, `output_tokens`, and `total_tokens`.
  - Finish reason maps to `Stop`, `Length`, `ToolCalls`, or `Other`.
- Error mapping:
  - Reuse shared HTTP helpers where possible.
  - Provider validation and protocol mismatches become non-retryable `ProtocolViolation` or `InvalidRequest`.
  - Transient HTTP/network failures remain retryable according to existing AI Gateway policy.

## Verification Plan

- Unit-test wire request mapping for a Cortex-like Primary turn with tools.
- Unit-test tool result replay mapping for `function_call_output`.
- Unit-test response parsing for text-only output.
- Unit-test response parsing for function calls.
- Unit-test response parsing for structured output and usage.
- Unit-test router/config acceptance of `openai_responses`.
- Run focused AI Gateway tests.

## Execution Notes

- key findings:
  - Cortex currently needs the complete path, not first-class streaming.
  - The first implementation can preserve Beluna-owned history by avoiding provider-managed continuation state.
  - Public API black-box tests fit the current Core test style better than adapter-private unit tests for this slice.
- decisions made:
  - Added a new `openai_responses` backend dialect.
  - Implemented Responses `complete(...)` only; `stream(...)` returns `UnsupportedCapability`.
  - Sent `store: false` and full Beluna Chat replay in each Responses request.
  - Mapped Responses `function_call` output items into existing `ToolCallResult` values and `function_call_output` replay items from Beluna tool-result history.
  - Mapped structured output through Responses `text.format`.
- final outcome:
  - Added OpenAI Responses backend adapter and black-box AI Gateway tests.
  - Verification passed:
    - `cargo fmt --all --check`
    - `cargo test -p beluna --test ai_gateway -- --nocapture`
    - `cargo test -p beluna --lib`
    - `cargo test -p beluna --tests --no-run`
