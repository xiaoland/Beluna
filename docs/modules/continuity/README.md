# Continuity Module

Code:
- `core/src/continuity/*`

Purpose:
- persist cognition state emitted by Cortex to local JSON storage
- maintain runtime capability overlay state (`new_neural_signal_descriptors` / `drop_neural_signal_descriptors`)
- provide deterministic per-act middleware gating (`on_act -> Continue|Break`)
- enforce cognition guardrails (immutable root partition, valid/unique goal numbering, goal weight in `[0,1]`)
- hold afferent-pathway sender for future continuity-generated senses

Non-scope:
- no cognition patch application API
- no spine event ingestion
