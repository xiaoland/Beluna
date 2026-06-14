# Continuity Store Implementation Plan

> Last Updated: 2026-06-14
> Status: implemented

## Scope

Implement Continuity as a generic durable record store and hard-cut the current
Continuity cognition-state API surface to the generic store boundary.

This slice should not implement Motor routine semantics, routine validation, or
Act-mediated store commands.

Implementation result:
[IMPLEMENTATION-RESULT.md](./IMPLEMENTATION-RESULT.md).

## Proposed Remaining Decisions

The main architectural decisions are now closed enough to implement. The
remaining choices can be treated as implementation defaults unless contradicted:

- Cognition namespace: `continuity.cognition`.
- Cognition record id: `state`.
- Cognition schema version: `cognition-state.v1`.
- Generic store envelope version: `1`.
- New record revision starts at `1`.
- `put_record` accepts an optional `expected_revision`; `None` means
  unconditional create-or-replace.
- `metadata` is deferred for MVP.
- `schema_version` is per record.
- `namespace + record_id` live in the map key, not duplicated inside the record.
- Record body is bytes plus content type, not `serde_json::Value`.
- Continuity exposes generic internal store methods first; Act-mediated commands
  remain later pathway work.
- Continuity removes `cognition_state_snapshot()` / `replace_cognition_state()`
  style APIs in this slice.
- Cortex owns cognition-state encode/decode/validation and uses generic
  Continuity record methods.
- OpenDAL itself is `ContinuityStorageBackend`; do not introduce a second
  Beluna-owned backend trait.
- `ContinuityPersistence` owns an OpenDAL blocking `Operator`, with fs service
  for runtime and memory service for tests if practical.
- Continuity promises single-process logical consistency; backend-level atomic
  write guarantees are not generalized yet.
- OpenDAL errors are normalized through `ContinuityError`, with a storage error
  variant if the existing shape is too narrow.

## Execution Steps

1. Inspect current Continuity code and tests again before editing.
2. Add generic store types:
   - `ContinuityStore`.
   - `ContinuityRecord`.
   - `ContinuityRecordKey`.
   - `ContinuityRecordBody`.
   - store envelope version / constants.
3. Migrate in-memory `ContinuityState` to own `ContinuityStore`.
4. Replace Continuity cognition-state APIs with generic record APIs:
   - `get_record(key)`.
   - `put_record(key, expected_revision, record/body/schema_version)`.
   - `delete_record(key, expected_revision)`.
   - optional namespace listing if needed by tests or Motor follow-up work.
5. Move cognition-state persistence usage to Cortex:
   - Cortex reads `continuity.cognition/state` via generic Continuity API.
   - Cortex decodes JSON bytes into `CognitionState`.
   - Cortex validates cognition shape before writing.
   - Cortex writes JSON bytes back through generic Continuity API.
6. Rework persistence:
   - introduce OpenDAL dependency.
   - make `ContinuityPersistence` own a blocking OpenDAL `Operator`.
   - treat the configured OpenDAL service/router/layers/operator as
     `ContinuityStorageBackend`.
   - serialize / deserialize the generic store envelope.
   - keep a migration hook shape, but implement it as no-op.
7. Update configuration / constructors:
   - preserve the existing path-based default constructor.
   - map the configured path to an OpenDAL fs root + object path.
8. Add focused tests:
   - generic record APIs can create, replace, read, and delete records.
   - cognition state persists and reloads through Cortex using generic
     Continuity records.
   - generic put/get/delete revision behavior.
   - persistence writes the new generic envelope.
   - record body accepts non-JSON bytes at the Continuity layer.
   - old cognition-only file migration is not required or tested.
9. Run targeted checks:
   - `cargo fmt --manifest-path core/Cargo.toml`.
   - `cargo test --manifest-path core/Cargo.toml continuity`.
   - `cargo check --manifest-path core/Cargo.toml`.

## Likely Files

- `core/Cargo.toml`.
- `core/src/continuity/types.rs`.
- `core/src/continuity/state.rs`.
- `core/src/continuity/persistence.rs`.
- `core/src/continuity/engine.rs`.
- `core/src/config/continuity.rs` if path mapping needs adjustment.
- `core/src/cortex/runtime/primary.rs`.
- existing Continuity tests, or new tests near the Continuity module.

## Non-Goals

- Motor routine source schema.
- Routine lifecycle Acts.
- Act-mediated Continuity store commands.
- Storage backend migration implementation.
- Data schema migration implementation.
- Legacy Continuity cognition-state compatibility APIs.
- Backward-compatible loading of old `cognition_state` files.
