# Current Continuity Reality

> Last Updated: 2026-06-14
> Status: evidence notes

## Current Files

- `core/src/continuity/engine.rs`
- `core/src/continuity/state.rs`
- `core/src/continuity/persistence.rs`
- `core/src/continuity/types.rs`
- `core/src/config/continuity.rs`

## Current Shape

Current Continuity persistence is specialized around `CognitionState`.

```text
ContinuityEngine
  state: ContinuityState
  persistence: ContinuityPersistence

ContinuityState
  cognition_state: CognitionState

ContinuityPersistence
  path: PathBuf
  load() -> Option<CognitionState>
  save(&CognitionState)
```

The persisted fs JSON envelope is currently cognition-specific:

```text
PersistedContinuityState {
  version,
  cognition_state
}
```

## Current Public-ish Operations

`ContinuityEngine` exposes:

- `with_defaults_at(path)`.
- `cognition_state_snapshot()`.
- `replace_cognition_state(state)`.
- `on_act(act, ctx)`.
- `flush()`.

`replace_cognition_state` validates cognition shape and immediately saves the
cognition state.

`flush` saves the current cognition state.

## Current Backend Coupling

`ContinuityPersistence` is directly fs JSON:

- creates parent directories.
- writes to a temporary file.
- renames temp file into place.
- reads JSON from a single configured path.

There is no storage backend trait yet.

## Migration Pressure

The new target is:

```text
ContinuityStore
  records: namespaces + records

backend
  load_store()
  save_store(store)
```

`CognitionState` should become a namespaced record instead of a direct
top-level persistence field.

Current compatibility decision:

- Old files containing only `cognition_state` do not need to be migrated for
  Issue 31.
- The architecture should still leave migration extension points for future
  storage backend and data schema migrations.
