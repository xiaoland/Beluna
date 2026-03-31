# AGENTS.md for core/src/cortex

`core/src/cortex` owns cognition cycle execution and internal cognition tooling.

## Local Role

- Cortex consumes `senses + physical_state`.
- Cortex emits control directives only; somatic act dispatch is mediated through tool-call-native paths.
- Cognition state is transformed inside Cortex and persisted/guardrailed through Continuity.

## High-Risk Invariants

- Runtime model:
  - Cortex is tick-driven only: one AI Gateway turn per admitted tick.
  - Empty-sense tick cycles are valid.
  - Primary accepts only assembled Input IR payload; no direct side channels are allowed.
  - If a tick turn returns tool calls, their results are buffered into the next admitted tick turn together with current tick input.

- Act and sense tooling:
  - Somatic act emission is tool-call-native; prompt text is not parsed to dispatch acts.
  - Dynamic act tools are created per turn with deterministic transport-safe aliases mapped back to fq act ids in runtime code.
  - Dispatch results come from the serial `Continuity -> Spine` pipeline and remain timeout-bounded.
  - `wait_for_sense` is valid only for acts with non-empty `might_emit_sense_ids`, and its gating is tick-skip based rather than afferent-rule mutation.

- Internal cognition tools:
  - Static tools are `expand-senses`, `add-sense-deferral-rule`, `remove-sense-deferral-rule`, `sleep`, and `patch-goal-forest`.
  - Sense references exposed to Primary use process-lifetime monotonic internal ids and deterministic rendered text.

- Goal-forest mutation:
  - Goal-forest patching goes through `goal_forest_helper` and produces a complete replacement `GoalNode[]`.
  - Goal hierarchy is nested through `children` arrays, not `parent_id/numbering` selectors.
  - Goal instincts live in Primary system prompt rather than a persisted root partition.

- Failure behavior:
  - Primary failure or timeout is fail-closed noop.
  - Helper failures must degrade through deterministic fallback rather than implicit recovery.

## Change Triggers

- Escalate to Product TDD if a change affects tick/coordination semantics, cross-unit authority, dispatch outcome semantics, or external contract shape.
- Update Core Unit TDD if a change alters cognition persistence assumptions, goal-forest mutation rules, or verification expectations.
