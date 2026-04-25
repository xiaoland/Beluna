# Issue #27 Cortex Runtime Skeleton Cleanup

> Last Updated: 2026-04-25
> Status: complete
> Scope: non-authoritative task packet for the Cortex runtime skeleton cleanup before issue #14
> Related issues: `#27`, `#14`

## MVT Core

- Objective & Hypothesis: Prepare Cortex runtime for the later Primary / Attention / Cleanup split by cleaning the route config naming and introducing narrow Primary session, tool, and executor boundaries, without implementing full Attention or Cleanup behavior yet.
- Guardrails Touched:
  - Cortex remains tick-driven and continues to dispatch somatic acts through tool-call-native paths.
  - The first slice must avoid new test work by user direction; verification relies on compile checks and focused review.
- Verification:
  - `cargo check --lib` succeeds after the cleanup.
  - Existing runtime behavior remains structurally reachable while the new boundaries are introduced, except for the explicitly confirmed `break-primary-phase` and same-tick multi-turn semantics.

## Scope

- In scope:
  - Replace the `helper_routes` concept with a route grouping that can later hold `primary / sense_helper / acts_helper / attention / cleanup`.
  - Start isolating Primary thread / continuation ownership behind a `PrimarySession` boundary.
  - Add the protocol skeleton for no-arg `break-primary-phase`.
- Out of scope:
  - Full Attention phase.
  - Full Cleanup phase.
  - Deterministic `patch-goal-forest.operations[]` reducer.
  - New tests.

## Notes

- This packet follows the implementation draft in `tasks/issue-14-cortex-three-chat-20260424/PLAN.md`.
- Keep the first code slice small enough that `primary.rs` does not become harder to reason about while moving responsibilities out.

## Result

- Replaced `CortexHelperRoutesConfig` with `CortexRoutesConfig`.
- Replaced `cortex.helper_routes` with `cortex.routes`.
- Removed route fields for `default` and `goal_forest_helper`.
- Added route fields for `attention` and `cleanup`; they are config surface only in this slice.
- Kept `GoalForestHelper` code present because deterministic `operations[]` reducer is a later slice.
- Added `PrimarySession` as the owner of Primary thread state and continuation state.
- Added `primary/tools.rs` for Primary static tool schema, tool argument DTOs, dynamic act tool binding, and dynamic act tool override construction.
- Added `primary/executor.rs` for `PrimaryToolExecutor` and tool execution side effects.
- Added no-arg `break-primary-phase` to Primary tools and system prompt.
- Added minimal fail-closed handling for duplicate `break-primary-phase`.
- Treat `break-primary-phase` as Primary phase completion even when the turn contains act tool calls.
- Updated local Cortex guardrails to reflect same-tick multi-turn Primary execution.
- Did not add tests by user direction.

## Follow-up Semantics Confirmed

- A pure assistant-text turn without tool calls is not a completed Primary phase.
- Runtime should continue Primary within the same admitted tick by opening another turn with a protocol reminder.
- The reminder should tell Primary to call `break-primary-phase` if the tick is complete, or continue reasoning / sense expansion / act emission if not.
- Same-tick Primary turns must have a max-turn guard to prevent unbounded self-talk.
- This intentionally changes the previous cortex invariant that allowed only one AI Gateway turn per admitted tick; promote the stable version to durable docs after the phase model lands.

## Follow-up Implementation Result

- Added `ReactionLimits.max_primary_turns_per_tick`, defaulting to 4.
- Changed Primary execution to loop within one admitted tick until `break-primary-phase` or max turn count.
- Tool-call continuation now continues immediately in the same admitted tick using the committed tool result history.
- Pure assistant-text turns without tools are allowed; runtime appends a protocol reminder as the next same-tick Primary turn.
- Exceeding `max_primary_turns_per_tick` fails the Primary phase closed for that tick.

## Completion Boundary

- `#27` completes the runtime skeleton cleanup needed before `#14`.
- Attention-like tools (`sleep`, sense deferral rules), cleanup-like tools (`patch-goal-forest`, `reset_context`), and `wait_for_sense` remain behaviorally in the Primary executor for now.
- Moving those responsibilities into Attention / Cleanup is `#14` implementation work, not `#27` skeleton cleanup.

## Verification Log

- `cargo fmt`
- `cargo check --lib`
- `cargo fmt`
- `cargo check --lib`
- `cargo fmt`
- `cargo check --lib`
