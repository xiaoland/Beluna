# Stem Runtime PRD

## Purpose

Stem is Beluna's runtime orchestrator for afferent sense processing and efferent act dispatch.

## Requirements

- One bounded Rust MPSC sense queue is the runtime ingress.
- No act queue exists; acts are dispatched inline and serially in Stem.
- Main process responsibilities:
  1. build queue and runtime components,
  2. start Stem loop,
  3. listen for SIGINT/SIGTERM.
- On shutdown:
  1. close ingress gate,
  2. block until `sleep` sense is enqueued,
  3. await stem completion and run cleanup.
- Control sense behavior:
  - `sleep`: stop loop, do not call Cortex.
  - `new_capabilities`: apply patch immediately, then call Cortex in same cycle.
  - `drop_capabilities`: apply drop immediately, then call Cortex in same cycle.
- Dispatch pipeline behavior:
  - order is Ledger -> Continuity -> Spine.
  - stage decision contract is `Continue` or `Break`.
  - `Break` cancels current act dispatch only.
- Capability patch conflicts use arrival-order-wins.

## Out of Scope

- Semantic planning policy.
- Long-term cognition memory model beyond current goal stack persistence.
