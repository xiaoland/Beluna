# Goal Management Contract

## Scope

Defines goal lifecycle behavior and active-goal invariants in `GoalManager`.

## Contract Cases

1. Given no active goal
- When a registered non-terminal goal is activated
- Then that goal becomes active and `active_goal_id` points to it.

2. Given an active goal exists
- When a different goal is activated without preemption
- Then activation is rejected with policy violation.

3. Given a merged goal source
- When it is re-activated
- Then activation is rejected.

4. Given a mid/low goal with unknown parent
- When it is registered
- Then registration is rejected as invalid request.

## Invariants

- At most one goal may be active.
- `active_goal_id` must reference an existing active record.
- `Merged` goals must have `merged_into` set.
- Terminal goals (`completed`, `cancelled`, `merged`) cannot transition back to active.
