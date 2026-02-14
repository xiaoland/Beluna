# Continuity LLD

Core structures:
- `ContinuityState`
- `ContinuityEngine`
- `ContinuityDispatchRecord`

State fields:
- `cognition_state`
- `capability_entries` (route -> descriptor)
- `tombstoned_routes`
- bounded dispatch record buffer

Determinism:
- patch/drop application follows arrival order.
- snapshot grouping uses stable ordered maps.
- dispatch event recording is append-only with bounded FIFO truncation.
