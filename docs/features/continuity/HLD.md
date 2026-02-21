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

- Root partition must match compile-time constants exactly.
- User partition root node id must be `user-root`.
- Goal node ids must be globally unique in user tree.
