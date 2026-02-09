# Delegation And Conflict Contract

## Scope

Defines delegation boundary and scoped conflict resolution behavior.

## Delegation Boundary

- Mind delegates through `DelegationCoordinatorPort` only.
- Port must not mutate `MindState` directly.
- Port does not own preemption or evolution policy.

## Conflict Ownership

`ConflictResolver` owns only:

1. helper output conflicts for the same intent,
2. evaluator verdict conflicts for the same criterion window,
3. merge compatibility conflicts.

## Contract Cases

1. Given helper outputs for the same intent
- When confidences differ
- Then highest-confidence helper result is selected.

2. Given helper confidence tie
- When conflict resolves
- Then lexical helper id tie-break applies.

3. Given evaluator verdict conflict
- When conflict resolves
- Then conservative verdict order applies (`fail > borderline > unknown > pass`).

4. Given merge compatibility conflict
- When conflict resolves
- Then deterministic `merge_allowed` or `merge_rejected` is emitted.
