# Continuity PRD

Continuity owns persisted operational state for runtime orchestration.

Invariants:

- Cognition persistence is deterministic and side-effect free outside Continuity boundaries.
- Capability patch/drop state is deterministic and replay-safe.
- Continuity does not perform semantic planning.

Requirements:

- Persist/retrieve `CognitionState`.
- Apply `new_capabilities` and `drop_capabilities` with arrival-order-wins policy.
- Provide capability snapshot contribution for physical state composition.
- Provide pre-dispatch gate decision (`Continue` / `Break`) and consume spine events.
