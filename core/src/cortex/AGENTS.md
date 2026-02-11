# AGENTS.md for core/src/cortex

Cortex owns goals and commitments, then emits non-binding `IntentAttempt[]` each cycle.

## Invariants
- Goal identity is separate from commitment lifecycle.
- Scheduling priority is dynamic and recomputed each cycle.
- `attempt_id` and `cost_attribution_id` derivation is deterministic.
