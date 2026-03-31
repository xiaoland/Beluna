# L3 Plan 05 - Goal Forest Reset and Thread Rewrite
- Task: `cortex-loop-architecture`
- Micro-task: `05-goal-forest-reset-and-thread-rewrite`
- Stage: `L3`
- Date: `2026-03-02`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Objective
Implement `patch-goal-forest(reset_context)` with AI Gateway-owned thread message rewrite and deterministic downstream sprout numbering generation.

## 2) Execution Steps
### Step 1 - Patch Tool Schema and Argument Parser
1. Update `core/src/cortex/primary.rs`:
- replace string `patch-goal-forest` input schema with object schema (`patch_instructions`, `reset_context`).
- replace string parser with typed args parser.
2. Keep strict no-compat mode (no legacy argument shapes).

### Step 2 - Gateway Thread Context Rewrite API
1. Add new public thread API in `core/src/ai_gateway/chat/api.rs`:
- `Thread::mutate_messages_atomically(request: ThreadMessageMutationRequest)`.
2. Add generic request/selector/update/outcome DTOs:
- `MessageBoundarySelector`, `MessageRangeSelector`, `SystemPromptUpdate`, `ThreadMessageMutationOutcome`.
3. Add store implementation in `core/src/ai_gateway/chat/store.rs` that atomically:
- resolves boundary selectors
- trims selected message range
- applies system prompt update in same lock scope.

### Step 3 - Thread-Scoped System Prompt Snapshot
1. Extend store `ThreadData` with `system_prompt_override`.
2. Extend `PreparedTurn` to include effective prompt snapshot.
3. Update `Thread::complete()`:
- use prepared prompt snapshot (thread override first, else chat default)
- prepend that system prompt as current behavior does.

### Step 4 - Deterministic Sprout Numbering Materialization
1. In `core/src/cortex/cognition_patch.rs`, extract explicit helper for sprout numbering resolution.
2. Ensure omitted numbering assignment is deterministic and order-preserving.
3. Keep numbering generation downstream from helper parse path.

### Step 5 - Hook Reset Flow into Primary Tool Execution
1. In `core/src/cortex/primary.rs`, after successful patch op application:
- if `reset_context=false`: current behavior.
- if `reset_context=true`:
  - render updated goal-forest section
  - build updated primary system prompt
  - call `thread.mutate_context_atomically(...)` with:
    - `trim_range: FirstUserMessage -> LatestAssistantToolBatchEnd`
    - `system_prompt_update: Replace(updated_prompt)`
2. Capture rewrite outcome in tool-result payload.

### Step 6 - Primary Micro-Loop Restart Semantics
1. Update micro-loop control flow in `run_primary_engine`:
- if a tool call applied `reset_context`, discard tool follow-up message chain.
- restart next step with a fresh user message for the same `primary_input`.
2. Keep cycle deadlines and max steps unchanged.

### Step 7 - Telemetry and Error Surface
1. Add debug/warn logs for:
- reset requested
- reset applied (`removed_messages`, `remaining_messages`)
- reset failed.
2. Keep failures non-panicking; tool returns `{ok:false}`.

## 3) File-Level Change Map
1. `core/src/cortex/primary.rs`
2. `core/src/cortex/helpers/goal_forest_helper.rs` (if parser/result payload changes are needed)
3. `core/src/cortex/cognition_patch.rs`
4. `core/src/cortex/prompts.rs`
5. `core/src/ai_gateway/chat/api.rs`
6. `core/src/ai_gateway/chat/store.rs`
7. `core/src/ai_gateway/chat/types.rs` (if shared DTO introduced)

## 4) Verification Gates
### Gate A - Tool Contract and Parser
```bash
rg -n "patch-goal-forest|reset_context|patch_instructions" core/src/cortex/primary.rs
```
Expected:
1. `patch-goal-forest` uses object schema with `reset_context`.
2. no string-only parser path remains.

### Gate B - Gateway Message Rewrite API
```bash
rg -n "mutate_messages_atomically|ThreadMessageMutationRequest|MessageBoundarySelector|SystemPromptUpdate|system_prompt_mode|PreparedTurn" core/src/ai_gateway/chat
```
Expected:
1. public thread rewrite API exists.
2. store has atomic rewrite path.
3. prepared turn carries effective system prompt snapshot.

### Gate C - Downstream Deterministic Numbering
```bash
rg -n "sprout|numbering|next_child_numbering|deterministic" core/src/cortex/cognition_patch.rs
```
Expected:
1. numbering generation remains downstream.
2. omitted numbering path is explicit and deterministic.

### Gate D - Micro-loop Reset Restart
```bash
rg -n "reset_context_applied|mutate_messages_atomically|primary_input" core/src/cortex/primary.rs
```
Expected:
1. reset path restarts next step from fresh user message.
2. tool-message chain is not reused after reset.

### Gate E - Build
Per workspace rule:
```bash
cd core && cargo build
cd ../cli && cargo build
```

## 5) Completion Criteria (05)
1. `patch-goal-forest` supports `reset_context`.
2. Thread rewrite operations are implemented in AI Gateway, not Cortex message store logic.
3. Reset path updates thread context and system prompt atomically.
4. Sprout numbering omissions are deterministically resolved downstream.
5. Core and CLI build successfully.

Status: `READY_FOR_REVIEW`
