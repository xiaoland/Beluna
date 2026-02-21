# Stem Runtime Contracts

Boundary:
- input: `Sense` queue (bounded mpsc)
- output: serial act dispatch side effects across Continuity -> Spine

Must hold:
- one bounded sense queue; no act queue.
- producer backpressure follows native bounded-channel blocking semantics.
- loop is tick-driven with configurable interval; default 1s.
- missed ticks are skipped.
- `hibernate` breaks loop immediately.
- `new_neural_signal_descriptors` / `drop_neural_signal_descriptors` are applied before same-cycle Cortex call.
- dispatch stage order is deterministic: Continuity gate -> Spine dispatch.
- pipeline decision contract is only `Continue` / `Break`.
- `Break` applies to current act only; next act continues.
- shutdown path closes ingress gate first and then blocks until `hibernate` is enqueued.
- sleep behavior is represented as Stem-provided act `core.control/sleep` with payload seconds.
