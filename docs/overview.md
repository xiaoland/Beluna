# Beluna Product Overview

Beluna is a survival-oriented digital life runtime, not a chatbot.

## Core Invariants

1. Natural language is the protocol across cognition and body affordances.
2. Cortex is stateless and pure at runtime boundary.
3. Cortex emits non-binding `EmittedAct[]` in `CortexOutput`.
4. Continuity persists cognition state (`goal-forest`) with deterministic guardrails.
5. Spine executes acts and can emit dispatch-failure senses.
6. Stem emits tick grants; Cortex runtime owns hybrid loop execution (tick + sense + act).
7. Body endpoints are Beluna's world interfaces.
8. Neural signal identity is descriptor-based (`sense_id` / `act_id`), while runtime envelopes use `sense_instance_id` / `act_instance_id`.
9. Proprioception is continuous internal state and is passed to Cortex as `<proprioception>` in Input IR.

## Runtime Topology

Beluna runtime process:
1. `core` runnable binary (`beluna`) with embedded standard body endpoints.
2. Optional external body endpoints (for example Apple Universal App) connect over UnixSocket.

Beluna Core top-level components:
1. Cortex (cognition)
2. Stem (tick/pathway/physical-state owner)
3. Continuity (operational cognition memory)
4. Ledger (resource control)
5. Spine (execution routing + dispatch-failure sense producer)

Operational flow:

```text
[BodyEndpoint, Spine, Continuity, Ledger] -> Afferent Pathway (bounded mpsc + deferral) -> Cortex Runtime
Stem:
  emits tick grants
  applies runtime control updates for ns_descriptor/proprioception
Cortex Runtime:
  drains afferent senses + tick grants -> Cortex Primary turn
  new_cognition_state -> Continuity persist
  emitted_acts -> Efferent Pathway (FIFO) -> serial dispatch worker: Continuity -> Spine
```

Runtime control:
1. Descriptor/proprioception mutations are direct runtime control calls (`Spine runtime -> StemControlPort`).
2. `hibernate` control sense and Stem sleep act dispatch are removed.

Shutdown flow:
1. `main` catches SIGINT/SIGTERM.
2. closes afferent ingress gate.
3. cancels runtime tasks.
4. waits for bounded efferent drain and cleanup.

## Runtime Logging

Core runtime logs are emitted through `tracing` only.

Default behavior:
1. JSON logs are written to `./logs/core` (unless `logging.dir` overrides).
2. Log file names follow `core.log.<YYYY-MM-DD>.<awake_sequence>`.
3. Retention cleanup removes historical log files older than `logging.retention_days` (default: 14).
4. `warn` and `error` are mirrored to stderr when `logging.stderr_warn_enabled=true`.
5. Log level/filter is configured via `logging.filter` (default: `info`).
