# L1 Plan 05 - Goal Forest Reset and Thread Rewrite
- Task: `cortex-loop-architecture`
- Micro-task: `05-goal-forest-reset-and-thread-rewrite`
- Stage: `L1`
- Date: `2026-03-02`
- Status: `DRAFT_FOR_APPROVAL`

## High-Level Strategy
1. Extend goal-forest patch tool with `reset_context: bool` as a first-class context-rewrite trigger.
2. Keep goal-forest op extraction in helper output (`Vec<GoalForestPatchOp>`), and make omitted sprout numbering deterministic in downstream apply/materialize stage.
3. Move thread rewrite mechanics into AI Gateway thread APIs so Cortex does not perform raw message-store surgery.
4. Keep rewrite transaction deterministic and atomic to avoid split state between thread history and system prompt.

## Architectural Design
1. Tool input contract becomes object-based:
- `patch_instructions: string`
- `reset_context: bool` (default `false`)
2. `reset_context=true` transaction (gateway-owned):
- derive updated goal-forest state from parsed patch ops
- trim messages from first user message through current assistant tool-call boundary
- replace goal-forest section in effective system prompt
- continue primary micro-loop with refreshed context
3. Failure handling:
- gateway reset path is one atomic mutation boundary
- no partial trim/prompt-update states are visible to Cortex.

## Key Technical Decisions
1. Thread mutation APIs are explicit AI Gateway operations, not ad-hoc rewriting in Cortex runtime code.
2. Goal-forest section source is canonical deterministic renderer only.
3. Reset flow orchestration remains in Cortex (tool semantics), but message-range trim + prompt mutation are encapsulated by a domain-agnostic AI Gateway atomic context mutation API.
4. Auditability requires reset telemetry with before/after message counts and updated prompt revision markers.

## Dependency Requirements
1. Micro-task `04` structured tool path is prerequisite.
2. AI Gateway chat/thread surface must support context-rewrite APIs used by Cortex Primary.
3. Micro-task `06` deterministic goal-forest renderer should be available before final cutover.

## L1 Exit Criteria
1. Reset transaction boundary and rollback policy are explicit.
2. Optional numbering behavior is contractually clear, with deterministic downstream fill for missing sprout numbering.
3. Thread rewrite and prompt replacement surfaces are defined on AI Gateway, not Cortex internals.
