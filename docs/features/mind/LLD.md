# Mind LLD

## Deterministic Loop Contract

`MindFacade::step` follows this strict order:

1. ingest command,
2. update state,
3. preemption (if new-goal competition),
4. delegation planning,
5. evaluation,
6. conflict resolution,
7. memory policy,
8. evolution decision,
9. output emission.

## Core Data Contracts

- `MindState`: goal map, active goal pointer, intent queue, recent evaluations/results, last preemption and memory directive.
- `MindCommand`: `ProposeGoal`, `ObserveSignal`, `SubmitDelegationResult`, `EvaluateNow`.
- `MindDecision`: preemption, delegation plan, evaluation, conflict, memory policy, evolution.
- `MindEvent`: lifecycle and stage-completion events.

## Invariants

- max one active goal,
- active pointer must match active record,
- merged goals require `merged_into`,
- confidence values clamped to `[0.0, 1.0]`,
- terminal goal states are immutable.

## Policy Rules

- preemption set is closed (`pause/cancel/continue/merge`),
- merge compatibility is deterministic,
- conflict resolver owns only scoped conflict classes,
- evolution is proposal-only and threshold-gated,
- low-confidence failures do not trigger evolution proposals.

## Contracts and Tests

Contracts:

- `docs/contracts/mind/goal-management.md`
- `docs/contracts/mind/preemption.md`
- `docs/contracts/mind/evaluation.md`
- `docs/contracts/mind/delegation-and-conflict.md`
- `docs/contracts/mind/evolution-trigger.md`
- `docs/contracts/mind/facade-loop.md`

Tests:

- `tests/mind/*`
- `tests/mind_bdt.rs`
