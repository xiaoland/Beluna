# AI Gateway Parallel Tool Calls — Implementation Plan

## Objective

Implement natively parallel tool calls with this ownership split:

1. **AI Gateway owns tool calling orchestration** (convert assistant `tool_calls` into function invocations and continuation control).
2. **Cortex keeps tool implementation logic** (actual behavior of each tool).
3. Add a **tool-call result continuation mode toggle**:
   - `immediate_new_turn`
   - `next_turn` (tick-aligned continuation)

## Why This Is a Large Change

Current code places tool execution and continuation state in Cortex Primary. Moving orchestration to AI Gateway changes module boundaries, thread state ownership, and continuation semantics.

## Required Trade-off Discussion (Before Coding)

To avoid readability/maintainability regression, one boundary decision must be explicit first:

- **Option A (recommended):** AI Gateway orchestrates via a generic executor interface; Cortex-specific state remains opaque to AI Gateway.
- **Option B:** AI Gateway directly understands Cortex tool semantics/state.

Option B creates tight coupling and degrades maintainability. This plan assumes **Option A**.

## Target Architecture

### A. Tool Executor Abstraction in AI Gateway

Add a chat-level executor contract (new module under `core/src/ai_gateway/chat/`):

- `ToolExecutor` trait (async)
- Input: assistant tool-call batch + executor context + policy
- Output: ordered tool-result frames mapped to `tool_call_id`

AI Gateway only orchestrates call scheduling and continuation, not tool semantics.

### B. Native Parallel Batch Execution

In AI Gateway tool scheduler:

- Execute one batch using parallel futures.
- Preserve deterministic output ordering by original assistant tool-call index.
- Enforce user-requested rule: **within one batch, the same tool name cannot appear twice**.

### C. Continuation Mode Toggle

Add an enum in chat turn config/state:

- `immediate_new_turn`: AI Gateway runs internal micro-loop (`assistant tool_calls -> execute -> tool messages -> next backend turn`) until terminal assistant text or max rounds.
- `next_turn`: AI Gateway executes tool calls now, stores generated tool-result messages in thread state, and waits for next caller turn; next turn prepends pending tool-result messages.

### D. Cortex Integration

- Extract current tool behavior from `run_primary_internal_tool_call(...)` into a Cortex executor implementation.
- CortexRuntime remains tick-driven.
- Cortex can set mode to `next_turn` for Primary thread to keep tick-admitted continuation.

## Implementation Phases

1. **Phase 0: Boundary finalization**
   - Finalize `ToolExecutor` trait and context/state handoff model.
   - Finalize duplicate-tool-name policy behavior (`InvalidRequest` on duplicate).

2. **Phase 1: AI Gateway orchestration scaffold**
   - Add tool executor module and scheduler.
   - Add continuation mode enum and wire it through `ChatOptions` / `ThreadOptions` / `TurnInput`.
   - Extend `ThreadStore` for pending tool-result messages required by `next_turn` mode.

3. **Phase 2: Cortex executor extraction**
   - Move tool implementation logic from `core/src/cortex/runtime/primary.rs` into a Cortex executor struct implementing AI Gateway `ToolExecutor`.
   - Keep existing behavior semantics while changing call site ownership.

4. **Phase 3: Parallel scheduler activation**
   - Enable parallel batch execution in AI Gateway.
   - Preserve deterministic tool-result message ordering.
   - Keep per-call telemetry (latency, result, error).

5. **Phase 4: Continuation mode behavior**
   - Implement `immediate_new_turn` loop in AI Gateway.
   - Implement `next_turn` pending-result buffering and next-turn injection.
   - Keep Cortex Primary on `next_turn` mode.

6. **Phase 5: Docs + validation**
   - Update cortex/ai-gateway contracts and module docs.
   - Run `cargo check` (no test run per workspace rule).

## Key Design Constraints

1. AI Gateway must remain backend/dialect-focused; tool orchestration must stay generic.
2. Cortex-specific mutable state should not leak into AI Gateway internals.
3. `next_turn` mode must remain tick-admitted for Cortex runtime behavior.
4. Duplicate tool names in one batch must be rejected deterministically.

## Risk Register

1. **State coupling risk:** moving orchestration can accidentally pull Cortex-specific state into AI Gateway.
   - Mitigation: opaque executor context/state token.
2. **Behavior drift risk:** immediate mode micro-loop can diverge from current tick semantics.
   - Mitigation: explicit mode toggle, Cortex fixed to `next_turn`.
3. **Ordering risk under parallelism:** side-effect tools may produce non-deterministic outcomes.
   - Mitigation: deterministic result ordering; if needed, add tool-level serial policy in executor.

## Deliverables

- `tasks/ai-gateway-parallel-tool-calls/CURRENT.md` (as-is architecture)
- AI Gateway tool executor abstraction and parallel scheduler
- Continuation mode toggle (`immediate_new_turn` / `next_turn`)
- Cortex executor implementation
- Updated contracts/docs
