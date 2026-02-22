# Stem Runtime LLD

## Queue and Shutdown

- One bounded `tokio::mpsc::channel<Sense>`.
- Afferent gateway:
  - `send(sense)` checks gate and applies backpressure.
  - `close_gate()` stops regular producers.
  - `send_hibernate_blocking()` bypasses gate check for guaranteed shutdown signal.
- `Sense::Hibernate` ends loop immediately.

## Tick Rules

- Interval source configured by `loop.tick_interval_ms`.
- Missed tick behavior configured by `loop.tick_missed_behavior` (`skip`).
- Tick can execute Cortex with empty domain senses.
- Default tick interval is `10000ms` (10s), configurable via `loop.tick_interval_ms`.
- If Primary output includes `<is-wait-for-sense>true</is-wait-for-sense>`, the next Active tick must wait until at least one sense arrives before invoking Cortex.
- If `<is-wait-for-sense>` is absent or false, Stem keeps normal tick-driven invocation.

## Sleep Act Rules

- Sleep act detection:
  - `endpoint_id == "core.control"`
  - `neural_signal_descriptor_id == "sleep"`
  - payload requires `seconds >= 1`
- On sleep act, Stem sets sleep deadline and stops dispatching remaining acts of current cycle.
- While sleeping:
  - new senses wake early and run cycle immediately
  - deadline expiry also triggers a cycle

## Dispatch Middleware Rules

- Per-act chain: `Continuity.on_act -> Spine.on_act`.
- `Break` aborts current act propagation only.
- Spine emits dispatch failure as domain sense through afferent sender.

## Capability Merge Rules

- Physical capability catalog is merged from:
  - Spine snapshot
  - Continuity overlay snapshot
  - Stem control descriptors
- Merge key: `(type, endpoint_id, neural_signal_descriptor_id)`.
- Final entries are sorted deterministically.
