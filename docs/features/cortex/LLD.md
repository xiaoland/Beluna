# Cortex LLD

## Deterministic IDs

- `cost_attribution_id = hash(cycle_id, commitment_id, goal_id, planner_slot)`
- `attempt_id = hash(cycle_id, commitment_id, goal_id, planner_slot, affordance_key, capability_handle, normalized_payload, requested_resources, cost_attribution_id)`

Both use canonical JSON and SHA-256 prefixing.

## Data Model

- `Goal`: semantic identity (`class`, `scope`, metadata/provenance).
- `CommitmentRecord`: operational pursuit (`status`, `created_cycle`, `last_transition_cycle`, `superseded_by`, `failure_code`).
- `SchedulingContext`: cycle-local dynamic priority.

## Transition Rules

- `ProposeGoal` registers goal only.
- `CommitGoal` creates active commitment.
- `SetCommitmentStatus` enforces terminal/failed constraints.
- `ObserveAdmissionReport` stores feedback signal.
