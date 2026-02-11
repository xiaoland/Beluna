# Non-Cortex LLD

## Determinism Rules

Admission purity:
- No wall clock.
- No randomness.
- No unordered iteration.
- Inputs limited to runtime state, attempt mechanical fields, and deterministic policies.

## Reservation Clock

- `expires_at_cycle = created_cycle + reservation_ttl_cycles`
- Expiration checked only against cycle clock.

## Settlement Idempotency

- Same `(reserve_entry_id, reference_id, terminal-op)` replay: no-op.
- Different reference on terminal reservation: invariant error.

## Attribution Chain

`cost_attribution_id` is carried from attempt -> admitted action -> spine event -> gateway telemetry -> external debit observation.
