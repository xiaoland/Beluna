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
- Cortex runtime is tick-driven only; sense arrivals are buffered and do not trigger immediate cycles.
- `wait_for_sense` uses tick-count suppression with correlated-sense matching in Cortex runtime.

## Sleep Act Rules

- Sleep act detection:
  - `endpoint_id == "core.control"`
  - `neural_signal_descriptor_id == "sleep"`
  - payload requires `ticks >= 1`
- Sleep suppresses admitted Cortex turns by tick count.
- While sleeping, senses continue buffering in afferent pathway/runtime queues.

## Dispatch Middleware Rules

- Stem enqueues non-sleep acts into one bounded dispatch queue.
- Dispatch worker is single-consumer serial.
- Per-act chain in worker: `Continuity.on_act -> Spine.on_act_final`.
- Dispatch terminal status is mapped to proprioception key:
  - `DISPATCHING`
  - `ACK | REJECTED | LOST`
- Terminal dispatch status keys are retained in bounded history; overflow uses `drop_proprioceptions`.

## Capability Merge Rules

- Physical capability catalog is merged from:
  - Spine snapshot
  - Continuity overlay snapshot
  - Stem control descriptors
- Identifier validation at Stem catalog boundary:
  - `endpoint_id` and `neural_signal_descriptor_id` must use dot-delimited ASCII alnum/dash segments (`[A-Za-z0-9-]+(\\.[A-Za-z0-9-]+)*`)
  - invalid patch/drop identifier entries are skipped with warning logs
- Merge key: `(type, endpoint_id, neural_signal_descriptor_id)`.
- Final entries are sorted deterministically.

## Proprioception Rules

- `main` startup proprioception is static base map for runtime lifetime.
- Dynamic proprioception is updated only by control senses:
  - `new_proprioceptions`
  - `drop_proprioceptions`
- Physical proprioception map passed to Cortex is:
  - startup base map
  - overlaid by dynamic map (last write wins by key)
- Only `Sense::Domain` entries are forwarded to Cortex; control senses are intercepted in Stem.
