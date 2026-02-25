# Continuity HLD

## Boundary

Inputs:
- full `CognitionState` from Cortex output
- capability patch/drop senses
- per-act middleware callback (`on_act`)

Outputs:
- cognition snapshot/persist API
- capability overlay catalog snapshot
- middleware decision (`Continue`/`Break`)

## Design

1. `ContinuityState` tracks:
   - cognition state
   - capability entries keyed by route
   - tombstoned routes
2. `ContinuityPersistence` handles JSON load/save with atomic replace.
3. `ContinuityEngine` is store + guardrail orchestrator.
4. `on_act` is currently no-op and deterministic `Continue`.

## Guardrails

- Goal numbering must be valid dotted-positive-integer paths (for example `1`, `1.2`, `3.4.5`).
- Goal numbering must be globally unique in goal-forest.
- Goal id must be non-empty and globally unique in goal-forest.
- Goal weight must be finite and in `[0,1]`.
- Goal `status` and `summary` must be non-empty.
