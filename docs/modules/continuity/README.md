# Continuity Module

Code:
- `core/src/continuity/*`

Purpose:
1. Persist cognition state emitted by Cortex to local JSON storage.
2. Validate cognition state invariants before persistence.
3. Provide deterministic per-act middleware gating (`on_act -> Continue|Break`).

Non-scope:
1. No descriptor/proprioception physical-state mutation ownership.
2. No Spine event ingestion pipeline.
3. No cognition patch transformation logic (owned by Cortex).
