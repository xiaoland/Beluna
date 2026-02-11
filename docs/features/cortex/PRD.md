# Cortex PRD

## Purpose

Cortex is Beluna's deliberative authority.

Cortex forms goals, manages commitments, decomposes work, and emits non-binding `IntentAttempt[]`.

## Requirements

- Goals are semantic identities (`Goal`) and do not encode runtime priority.
- Commitments are operational pursuit records (`CommitmentRecord`) with lifecycle and `created_cycle`.
- Scheduling priority is dynamic per cycle, not stored on `Goal`.
- `attempt_id` and `cost_attribution_id` are deterministic.
- Cortex can intend anything; execution is constrained downstream.

## Out of Scope

- Direct execution access.
- Constraint narration or moral interpretation.
