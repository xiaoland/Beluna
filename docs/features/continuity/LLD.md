# Continuity LLD

Core structures:
- `ContinuityState`
- `ContinuityEngine`
- `ContinuityPersistence`

Persisted JSON payload:
- `version`
- `cognition_state`

State fields:
- `cognition_state`
- `neural_signal_descriptor_entries` (route -> descriptor)
- `tombstoned_routes`
- `neural_signal_descriptor_version`

APIs:
- `cognition_state_snapshot()`
- `persist_cognition_state(state)`
- `apply_neural_signal_descriptor_patch(patch)`
- `apply_neural_signal_descriptor_drop(drop_patch)`
- `neural_signal_descriptor_snapshot()`
- `on_act(act, ctx) -> Continue|Break`

Determinism:
- patch/drop application follows arrival order.
- snapshot ordering is stable by `(type, endpoint_id, descriptor_id)`.
- cognition persistence is validated before write.
- JSON persistence uses temp-file write + sync + rename.
