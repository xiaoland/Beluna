# AGENTS.md for core/src/cortex

Cortex is a stateless cognition boundary that consumes a drained sense batch plus physical/cognition snapshots and emits `Act[]` + next cognition state.

## Invariants
- Progression is input-event driven only.
- Cortex does not durably persist cognition/goal state internally.
- Primary output is prose IR; sub-stages compile to structured drafts.
- Deterministic clamp is final authority before act emission.
- `act_id` derivation is deterministic.
