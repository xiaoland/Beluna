# Continuity Store Implementation Result

> Last Updated: 2026-06-14
> Status: implemented in Core; durable docs still pending

## Implemented

- Added generic Continuity store types:
  - `ContinuityStore`.
  - `ContinuityRecord`.
  - `ContinuityRecordKey`.
  - `ContinuityRecordBody`.
- Represented record body as `content_type + bytes`, not `serde_json::Value`.
- Moved `ContinuityState` from direct `CognitionState` ownership to generic
  `ContinuityStore` ownership.
- Removed Continuity cognition-state APIs such as
  `cognition_state_snapshot()` / `persist_cognition_state()`.
- Added generic Continuity engine operations:
  - `get_record`.
  - `put_record`.
  - `delete_record`.
- Moved cognition-state encode/decode/validation ownership into Cortex.
- Persisted Cortex cognition state as:

```text
namespace = continuity.cognition
record_id = state
schema_version = cognition-state.v1
content_type = application/json
```

- Added OpenDAL `0.57.0` as the Continuity storage backend layer.
- `ContinuityPersistence` now owns an OpenDAL blocking `Operator`.
- The configured path-based constructor remains intact:
  `ContinuityEngine::with_defaults_at(path)`.
- The first persistence layout remains a single JSON store document at the
  configured path; OpenDAL owns the storage service/operator boundary.
- No old cognition-only file migration was implemented.

## Runtime Notes

OpenDAL `blocking::Operator` wraps an async `Operator` and needs a Tokio runtime
handle. Continuity handles both call shapes:

- construction inside Tokio runtime captures the current handle.
- construction outside Tokio runtime uses a small internal runtime.
- storage calls made from a Tokio async context are moved to an OS thread before
  invoking the blocking operator, avoiding direct nested `block_on` on an async
  worker thread.

## Verification

Passed:

```bash
cargo fmt --manifest-path core/Cargo.toml
cargo check --manifest-path core/Cargo.toml
cargo test --manifest-path core/Cargo.toml continuity --lib
cargo test --manifest-path core/Cargo.toml cognition_state_persists_through_generic_continuity_record --lib
cargo test --manifest-path core/Cargo.toml --lib --bins
```

Coverage added:

- generic record create / replace / read / delete revision behavior.
- non-JSON byte body support at the Continuity store layer.
- OpenDAL-backed persistence reload and new JSON envelope shape.
- Cortex cognition persistence through generic Continuity records.

Full `cargo test --manifest-path core/Cargo.toml` still fails at the Agent Task
harness boundary because AIMock does not become ready:

```text
core.intent_to_act_ack.v1 errored: AIMock did not become ready at http://127.0.0.1:<port>/health
core.shell_write_file_world_diff.v1 errored: AIMock did not become ready at http://127.0.0.1:<port>/health
```

This matches the pre-existing Agent Task harness readiness failure and is not a
Continuity store unit failure.
