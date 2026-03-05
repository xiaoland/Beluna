# Stem Runtime PRD

## Purpose

Stem is Beluna's tick-driven runtime orchestrator.

## Requirements

- Runtime loop is driven by interval ticks, not only new incoming senses.
- Default tick interval is `1s`.
- Missed ticks are skipped.
- One bounded MPSC sense queue is the afferent ingress.
- One bounded async dispatch queue exists; dispatch worker stays serial and ordered.
- `hibernate` is shutdown control sense and terminates the loop.
- Capability control senses:
  - `new_neural_signal_descriptors`: apply before same-cycle Cortex call
  - `drop_neural_signal_descriptors`: apply before same-cycle Cortex call
- Stem validates descriptor identifiers before committing to catalog:
  - `endpoint_id` and `neural_signal_descriptor_id` must match `[A-Za-z0-9-]+(\\.[A-Za-z0-9-]+)*`
  - invalid descriptor patch/drop routes are rejected from catalog updates
- Proprioception control senses:
  - `new_proprioceptions`: apply map upsert before same-cycle Cortex call
  - `drop_proprioceptions`: apply key drop before same-cycle Cortex call
- Stem publishes a built-in sleep act descriptor:
  - endpoint: `core.control`
  - act id: `sleep`
  - payload: `{ "ticks": integer >= 1 }`
- Sleep act semantics:
  - suppresses admitted Cortex turns for the requested number of ticks
  - senses keep buffering while suppressed
- Dispatch path is per act middleware:
  - `Continuity.on_act -> Spine.on_act_final`
  - final status is `ACK | REJECTED | LOST`
  - dispatch status is exposed as Stem proprioception key `stem.dispatch.<act_instance_id>.status`

## Out of Scope

- Semantic planning policy.
- Ledger settlement path in dispatch chain.
