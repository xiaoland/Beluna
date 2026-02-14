# Continuity Module

Code:
- `core/src/continuity/*`

Purpose:
- persist cognition state emitted by Cortex
- maintain runtime capability overlay state (`new_capabilities` / `drop_capabilities`)
- provide deterministic pre-dispatch gating contract (`Continue` / `Break`)
- ingest spine events for continuity-side operational records
