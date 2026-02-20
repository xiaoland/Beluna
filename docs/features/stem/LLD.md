# Stem Runtime LLD

## Queue + Shutdown Rules

- One bounded `tokio::mpsc::channel<Sense>`.
- Producer sends go through `SenseIngress::send`.
- `close_gate()` rejects future producer sends.
- `send_sleep_blocking()` bypasses gate check and blocks on queue backpressure.

## Control Sense Rules

- `sleep`: immediate loop break; Cortex not called.
- `new_capabilities`: apply patch before same-cycle Cortex call.
- `drop_capabilities`: apply drop before same-cycle Cortex call.

## Dispatch Rules

- Serial dispatch order: Ledger -> Continuity -> Spine.
- Pipeline decision contract: `Continue` / `Break`.
- `Break` aborts current act only.
- Spine errors map to deterministic synthetic rejection events.

## Ledger Settlement Rules

- Deterministic `cost_attribution_id` from `(cycle_id, act_id)`.
- Reservation terminality enforced by ledger (`settle|refund|expire` exactly one).
- Settlement idempotent by `(reserve_entry_id, reference_id, terminal-op)`.
