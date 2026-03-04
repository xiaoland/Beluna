# Stem Runtime Contracts

Boundary:
1. Stem owns pathway construction and physical-state mutation.
2. Stem emits tick grants and hosts the efferent serial dispatch worker.

Must hold:
1. Afferent pathway is bounded MPSC with backpressure, deferral scheduler, and observe-only sidecar.
2. Cortex owns afferent consumption; Stem must never invoke Cortex directly.
3. Tick runtime emits `TickGrant` at configured interval (default 10s) with missed behavior `skip`.
4. Physical state mutation (`ns_descriptor`, proprioception) is controlled through `StemControlPort`; ns_descriptor mutation returns accepted/rejected commit results.
5. Efferent dispatch order is deterministic and serial per queue order:
- stage 1: Continuity `on_act`
- stage 2: Spine `on_act_final`.
6. Continuity `Break` only short-circuits current act.
7. Dispatch status proprioception is emitted as `DISPATCHING -> ACK|REJECTED|LOST` with bounded retention.
8. Shutdown is bounded:
- close afferent ingress gate
- cancel runtimes
- efferent worker drains queued acts until timeout.
