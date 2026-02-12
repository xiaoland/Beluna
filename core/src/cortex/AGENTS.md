# AGENTS.md for core/src/cortex

Cortex is an always-on reactor that consumes bounded `ReactionInput` and emits non-binding `IntentAttempt[]`.

## Invariants
- Reactor progression is inbox-event driven only.
- Cortex does not durably persist goals/commitments.
- Primary output is prose IR; sub-stages compile to structured attempts.
- Deterministic clamp is final authority before attempt emission.
- `attempt_id` and `cost_attribution_id` derivation is deterministic.
