# Stem Runtime HLD

## Pipeline

1. Tick fires from interval scheduler.
2. Drain queued senses non-blocking.
3. Apply capability patch/drop senses to Continuity overlay.
4. Apply proprioception patch/drop senses to Stem dynamic proprioception map.
5. Compose `PhysicalState` from descriptor catalogs + merged proprioception map.
5. Load cognition snapshot from Continuity.
6. Invoke Cortex with domain senses (may be empty).
7. Persist full `new_cognition_state` into Continuity.
8. Enqueue acts to async serial dispatch worker.
9. Dispatch worker processes each act through middleware:
   - `Continuity.on_act`
   - `Spine.on_act_final`
10. Intercept Stem-provided sleep act and switch scheduler mode.

## Scheduler Modes

- `Active`: interval tick-driven execution.
- `SleepingUntil(deadline)`: wait for deadline or early wake by new sense.

## Communication Model

- Afferent-Pathway (sense queue):
  - implementation: bounded `tokio::mpsc::Sender<Sense>`
  - producers: body endpoints via Spine adapters, Spine runtime failure emission, Continuity (reserved), Main shutdown
  - consumer: Stem loop only
- Efferent-Pathway (act dispatch):
  - implementation: one bounded queue + one serial worker + Spine endpoint routing
  - producer: Stem from Cortex output acts
  - consumers: Continuity middleware then Spine router/endpoints
