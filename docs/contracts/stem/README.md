# Stem Runtime Contracts

Boundary:
- input: `Sense` queue (bounded mpsc)
- output: async serial act dispatch side effects across Continuity -> Spine

Must hold:
- one bounded sense queue; no act queue.
- producer backpressure follows native bounded-channel blocking semantics.
- loop is tick-driven with configurable interval; default 1s.
- missed ticks are skipped.
- after any non-tick-triggered cycle (`wait_for_sense` wake or sleep wake), interval schedule is reset so the next Active tick waits a full period.
- `hibernate` breaks loop immediately.
- `new_neural_signal_descriptors` / `drop_neural_signal_descriptors` are applied before same-cycle Cortex call.
- `new_proprioceptions` / `drop_proprioceptions` are applied before same-cycle Cortex call.
- domain senses sent to Cortex exclude control senses (`hibernate`, descriptor patch/drop, proprioception patch/drop).
- dispatch stage order is deterministic within dispatch worker: Continuity gate -> Spine dispatch.
- dispatch worker keeps ordered serial processing (`DISPATCHING -> ACK|REJECTED|LOST`).
- shutdown path closes ingress gate first and then blocks until `hibernate` is enqueued.
- sleep behavior is represented as Stem-provided act `core.control/sleep` with payload seconds.
