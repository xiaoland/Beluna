# Continuity PRD

Continuity owns deterministic persistence and guardrails for runtime cognition state.

## Requirements

- Persist/retrieve full `CognitionState` as JSON on local disk.
- Validate cognition guardrails before accepting persisted or runtime updates.
- Keep capability overlay state from:
  - `new_neural_signal_descriptors`
  - `drop_neural_signal_descriptors`
- Provide capability snapshot contribution for Stem physical-state composition.
- Participate in per-act middleware dispatch with `on_act -> Continue|Break`.
- Hold afferent-pathway sender for future continuity-generated senses.

## Non-Requirements

- No semantic planning.
- No patch application API (patches are applied inside Cortex).
- No spine event ingestion or act execution record tracking.
