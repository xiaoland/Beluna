# Stem Runtime Contracts

Boundary:
- input: `Sense` queue (bounded mpsc)
- output: serial act dispatch and settlement side effects across Ledger -> Continuity -> Spine

Must hold:
- one bounded sense queue; no act queue.
- producer backpressure follows native bounded-channel blocking semantics.
- `sleep` breaks loop immediately and skips Cortex.
- `new_capabilities` / `drop_capabilities` are applied before same-cycle Cortex call.
- dispatch stage order is deterministic: Ledger pre-dispatch -> Continuity gate -> Spine dispatch -> settlement callbacks.
- pipeline decision contract is only `Continue` / `Break`.
- `Break` applies to current act only; next act continues.
- shutdown path closes ingress gate first and then blocks until `sleep` is enqueued.
- ledger reservation terminality and idempotent settlement by reference are preserved.
