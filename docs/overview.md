# Beluna Product Overview

Beluna is a survival-oriented digital life runtime, not a chatbot.

## Core Invariants

1. Natural language is the protocol across cognition and body affordances.
2. Cortex is stateless and pure at runtime boundary.
3. Cortex emits non-binding `Act[]`.
4. Continuity persists cognition state (`goal-tree` + `l1-memory`) with deterministic guardrails.
5. Spine executes acts and can emit dispatch-failure senses.
6. Stem loop is tick-driven (default 1s, missed tick skip).
7. Body endpoints are Beluna's world interfaces.

## Runtime Topology

Beluna runtime process:
1. `core` runnable binary (`beluna`) with embedded standard body endpoints.
2. Optional external body endpoints (for example Apple Universal App) connect over UnixSocket.

Beluna Core top-level components:
1. Cortex (cognition)
2. Stem (runtime loop/orchestrator)
3. Continuity (operational memory + capability overlay)
4. Ledger (resource control, currently short-circuited from Stem dispatch path)
5. Spine (execution routing + dispatch-failure sense producer)

Operational flow:

```text
[BodyEndpoint, Spine, Continuity, Ledger] -> SenseQueue (bounded mpsc) -> Stem
Stem:
  drained senses -> compose(physical_state + cognition_state) -> Cortex
  Cortex -> (acts, new_cognition_state)
  new_cognition_state -> Continuity persist
  acts -> serial dispatch: Continuity -> Spine
```

Control senses:
1. `hibernate`: stops Stem loop.
2. `new_neural_signal_descriptors`: applies patch before same-cycle Cortex call.
3. `drop_neural_signal_descriptors`: applies drop patch before same-cycle Cortex call.

Shutdown flow:
1. `main` catches SIGINT/SIGTERM.
2. closes ingress gate (rejects producer sends).
3. blocks until `hibernate` is enqueued.
4. waits for Stem completion and runs cleanup.

## Runtime Logging

Core runtime logs are emitted through `tracing` only.

Default behavior:
1. JSON logs are written to `./logs/core` (relative to process current working directory unless `logging.dir` overrides).
2. File rotation defaults to daily (`logging.rotation` supports `daily` and `hourly`).
3. Retention cleanup removes prefixed historical log files older than `logging.retention_days` (default: 14).
4. `warn` and `error` are mirrored to stderr when `logging.stderr_warn_enabled=true` (default).
5. Log level/filter is configured via `logging.filter` (default: `info`).
