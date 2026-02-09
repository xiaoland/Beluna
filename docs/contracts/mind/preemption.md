# Preemption Contract

## Scope

Defines preemption decision outputs and safe-point constraints.

## Contract Cases

1. Given a non-preemptable safe point
- When a new goal arrives
- Then disposition is `continue`.

2. Given a preemptable safe point and higher-priority incoming goal
- When merge compatibility is false
- Then disposition is `pause`.

3. Given repeated reliability failures on active goal
- When a preemptable new goal arrives
- Then disposition is `cancel`.

4. Given merge-compatible active and incoming goals
- When preemption runs
- Then disposition is `merge` with deterministic merged goal id.

5. Given `preemptable=false`
- When a checkpoint token is present
- Then decision is rejected as policy violation.

## Disposition Set

The output set is closed and explicit:

- `pause`
- `cancel`
- `continue`
- `merge`
