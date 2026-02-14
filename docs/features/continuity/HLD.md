# Continuity HLD

Boundary:
- Inputs: capability patch/drop senses, cognition state updates, dispatch callbacks.
- Outputs: cognition snapshot/persist API, capability snapshot, pre-dispatch decisions.

Design:
1. State container tracks:
   - `cognition_state`
   - capability entries by route key
   - tombstoned routes
   - dispatch event records
2. Patch/drop updates mutate capability state in receive order.
3. Capability snapshot is grouped by endpoint and exposed as Cortex-facing catalog shape.
4. Dispatch callbacks are mechanical and deterministic.
