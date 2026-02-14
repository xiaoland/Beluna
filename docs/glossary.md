# Product Glossary

- Cortex: Stateless cognition boundary `cortex(sense, physical_state, cognition_state) -> (acts, new_cognition_state)`.
- Stem: Runtime loop that consumes senses, invokes cortex, persists cognition, and dispatches acts serially.
- Continuity: Operational state owner for cognition persistence and capability overlay patch/drop.
- Ledger: Survival budget subsystem that reserves, settles, refunds, expires, and records debits.
- Spine: Transport-agnostic execution substrate that routes acts and emits ordered settlement events.
- Sense: Canonical ingress datum consumed by Stem (`domain`, `sleep`, `new_capabilities`, `drop_capabilities`).
- Sense Queue: Bounded Rust MPSC queue shared by all afferent producers.
- Capability Patch: Incremental upsert payload for capability catalog updates.
- Capability Drop Patch: Incremental removal payload by route key.
- Physical State: Current ledger snapshot + merged capabilities visible to Cortex.
- Cognition State: Persisted goal stack and future cognitive state extensions.
- Act: Non-binding executable proposal emitted by Cortex and dispatched by Stem.
- Dispatch Decision: Pipeline control signal (`Continue` or `Break`) for current act only.
- Route Key: Composite (`endpoint_id`, `capability_id`) routing identity in Spine.
- Reservation Terminality: Exactly one terminal transition per reservation (`Settled`, `Refunded`, `Expired`), idempotent by reference.
