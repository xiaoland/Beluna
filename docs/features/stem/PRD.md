# Stem Runtime PRD

## Purpose

Stem is Beluna's tick-driven runtime orchestrator.

## Requirements

- Runtime loop is driven by interval ticks, not only new incoming senses.
- Default tick interval is `1s`.
- Missed ticks are skipped.
- One bounded MPSC sense queue is the afferent ingress.
- No act queue exists; acts are dispatched inline and serially per cycle.
- `hibernate` is shutdown control sense and terminates the loop.
- Capability control senses:
  - `new_neural_signal_descriptors`: apply before same-cycle Cortex call
  - `drop_neural_signal_descriptors`: apply before same-cycle Cortex call
- Stem publishes a built-in sleep act descriptor:
  - endpoint: `core.control`
  - act id: `sleep`
  - payload: `{ "seconds": integer >= 1 }`
- Sleep act semantics:
  - enters sleeping mode until deadline
  - new sense arrival wakes early and triggers immediate cycle
- Dispatch path is per act middleware:
  - `Continuity.on_act -> Spine.on_act`
  - each stage returns `Continue|Break` for current act only

## Out of Scope

- Semantic planning policy.
- Ledger settlement path in dispatch chain.
