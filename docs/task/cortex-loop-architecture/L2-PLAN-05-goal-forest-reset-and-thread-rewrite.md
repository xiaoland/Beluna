# L2 Plan 05 - Goal Forest Reset and Thread Rewrite
- Task: `cortex-loop-architecture`
- Micro-task: `05-goal-forest-reset-and-thread-rewrite`
- Stage: `L2`
- Date: `2026-03-02`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Goal and Scope
Goal:
1. Add `reset_context` semantics to `patch-goal-forest`.
2. Implement thread-message rewrite through AI Gateway thread APIs (not Cortex-side message store mutation).
3. Guarantee deterministic numbering generation for `sprout` ops when numbering is omitted.

In scope:
1. Patch-goal-forest tool schema and runtime behavior changes.
2. AI Gateway thread/store API extensions for reset transaction.
3. Primary micro-loop control-flow changes needed after context reset.

Out of scope:
1. Efferent pipeline redesign (`07`).
2. State ownership cleanup (`06`).

## 2) Tool Contract Freeze (`patch-goal-forest`)
Replace string argument with strict object schema:

```json
{
  "type": "object",
  "properties": {
    "patch_instructions": { "type": "string", "minLength": 1 },
    "reset_context": { "type": "boolean", "default": false }
  },
  "required": ["patch_instructions"],
  "additionalProperties": false
}
```

Behavior:
1. `patch_instructions` is still converted into `Vec<GoalForestPatchOp>` by helper sub-agent.
2. `reset_context=false`: patch only the cycle-local goal forest.
3. `reset_context=true`: patch goal forest, then invoke AI Gateway context rewrite transaction.

## 3) Deterministic Numbering Stage (Downstream)
Contract:
1. Helper output remains `Vec<GoalForestPatchOp>`.
2. Omitted `sprout.numbering` is resolved downstream, not in parsing/prompt layer.

Deterministic policy:
1. For each `sprout` op in order, resolve parent by existing selector rules.
2. If numbering omitted, assign `next = max(existing_direct_child_index)+1` under resolved parent.
3. Numbering is never compacted or renumbered after `prune`.
4. Same initial node set + same ordered ops => same resulting numbering.

Placement:
1. Keep canonical logic in `cortex::cognition_patch` path (single source of truth).
2. Expose explicit helper (for readability) that materializes sprout numbering before insert/apply.

## 4) AI Gateway Message Rewrite API (Owned by Gateway)
Public API shape (`chat::api`):
1. `Thread::mutate_messages_atomically(request: ThreadMessageMutationRequest) -> Result<ThreadMessageMutationOutcome, GatewayError>`

Request (domain-agnostic):
1. `trim_range: Option<MessageRangeSelector>`
2. `system_prompt_update: SystemPromptUpdate`

`MessageRangeSelector` (chat-domain selectors):
1. `start: MessageBoundarySelector`
2. `end: MessageBoundarySelector`

`MessageBoundarySelector`:
1. `FirstUserMessage`
2. `LatestAssistantToolBatchEnd`

`SystemPromptUpdate`:
1. `Keep`
2. `Replace(String)`
3. `Clear`

Outcome:
1. `removed_messages: usize`
2. `remaining_messages: usize`
3. `effective_system_prompt_changed: bool`

Store-level internal API (`chat::store`):
1. `mutate_thread_messages_atomically(chat_id, thread_id, request) -> Result<ThreadMessageMutationOutcome, GatewayError>`

Thread rewrite invariants:
1. Start boundary: first stored `User` message.
2. End boundary: latest assistant message with non-empty `tool_calls`, plus contiguous following `Tool` messages linked by `tool_call_id`.
3. Deletes inclusive range `[start..=end]` atomically under one write lock.
4. Updates thread-effective system prompt in the same atomic operation.
5. On boundary resolution failure, returns `InvalidRequest` without mutation.

System prompt ownership:
1. Effective system prompt becomes thread-scoped (fallback to chat-level default if not overridden).
2. `prepare_turn` returns effective system prompt snapshot for that turn.

## 5) Cortex Primary Runtime Contract Changes
Patch tool execution result extends with:
1. `reset_context_applied: bool`
2. `rewrite_outcome` metadata when reset succeeds.

Micro-loop behavior:
1. When `reset_context_applied=false`: continue existing tool-message loop as today.
2. When `reset_context_applied=true`:
- do not continue with tool-role follow-up messages bound to removed assistant tool call.
- restart next micro-turn with fresh user prompt (`primary_input`) on same persistent thread.
- next turn consumes updated system prompt snapshot from AI Gateway.

## 6) Error and Telemetry Contracts
Error mapping:
1. Gateway rewrite failure => tool result `{ok:false,...}` and no partial local/global mutation mismatch.
2. Local goal-forest patch failure => no rewrite attempt.

Telemetry:
1. `goal_forest_reset_context_requested`
2. `goal_forest_reset_context_applied`
3. `goal_forest_reset_context_failed`
4. include `removed_messages`, `remaining_messages`, and `cycle_id`.

## 7) File/Interface Freeze for L3
1. `core/src/cortex/primary.rs`
2. `core/src/cortex/helpers/goal_forest_helper.rs`
3. `core/src/cortex/cognition_patch.rs`
4. `core/src/cortex/prompts.rs`
5. `core/src/ai_gateway/chat/api.rs`
6. `core/src/ai_gateway/chat/store.rs`
7. `core/src/ai_gateway/chat/types.rs` (if outcome type shared)
8. `docs/modules/ai-gateway/*` and `docs/modules/cortex/*` (contract refresh in `08`, but touched surfaces listed now)

## 8) Risks and Constraints
1. Resetting thread history can invalidate pending tool-message chains.
Mitigation: explicit micro-loop restart path after successful reset.
2. Thread-scoped mutable system prompt introduces new state axis.
Mitigation: prompt snapshot captured in `prepare_turn`, immutable for in-flight turn.
3. Deterministic numbering may still fail on ambiguous parent selectors.
Mitigation: fail op deterministically with clear tool error payload.

## 9) L2 Exit Criteria (05)
1. `patch-goal-forest` schema with `reset_context` is frozen.
2. AI Gateway message rewrite API boundary is frozen and Cortex does not mutate thread messages directly.
3. Downstream deterministic sprout numbering contract is frozen.
4. Primary micro-loop reset restart behavior is frozen.

Status: `READY_FOR_REVIEW`
