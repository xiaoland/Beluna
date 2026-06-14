# Continuity Store Open Decisions

> Last Updated: 2026-06-14
> Status: mostly closed; remaining choices are implementation defaults unless
> contradicted

## Store Placement

Confirmed:

```text
PersistedContinuityState {
  records: ContinuityRecordStore
}
```

`CognitionState` is represented as a store namespace rather than a sibling
field.

Implementation default:

- Singleton cognition namespace: `continuity.cognition`.
- Singleton cognition record id: `state`.
- Hard cut-off: remove Continuity APIs such as `cognition_state_snapshot()` /
  `replace_cognition_state()` instead of keeping compatibility wrappers.
- Cortex reads/writes its cognition record through the generic Continuity store
  API and owns cognition-specific encode/decode/validation.

## Record Envelope

Implementation default:

```text
ContinuityRecord {
  revision: u64,
  schema_version: String,
  body: ContinuityRecordBody
}

ContinuityRecordBody {
  content_type: String,
  bytes: Vec<u8>
}
```

Confirmed:

- `metadata` is deferred for MVP.
- `schema_version` is per record.
- `namespace + record_id` live in the map key, not duplicated inside the record.
- The domain record body should not be `serde_json::Value`; JSON is one codec,
  not the durable-store contract.
- The MVP can encode current cognition state and routine source as UTF-8 JSON or
  UTF-8 text bytes, but the record contract should also support non-JSON bytes.

## Revision Semantics

Implementation default:

```text
put_record(namespace, record_id, expected_revision?, body)
delete_record(namespace, record_id, expected_revision?)
```

Confirmed:

- Compare-and-set is supported but not required for every write.
- `expected_revision = None` means unconditional create-or-replace.
- Newly created records start at revision `1`.

## API Surface

Confirmed:

- implement internal port first to keep the store boundary simple.
- later let Motor lifecycle Acts call into Motor, and Motor can use the
  internal store port or emit Continuity Acts depending on the Motor design.
- Issue 31 does not require all learned knowledge persistence to be Neural
  Signal-visible from day one.

## Namespace Validation

Confirmed:

- generic shape validation only for MVP.
- Motor validates routine source semantics before activation.
- Cortex owns cognition-state validation before writing its cognition record.

## Persistence Compatibility

Confirmed:

- No old `cognition_state` file migration is required for this issue.
- Architecture should still leave a migration hook for future storage backend
  migration and data schema migration.

## Storage Backend Boundary

Confirmed:

- Continuity should leave architecture space for storage backends beyond the
  current fs JSON implementation.
- Storage backend abstraction is about physical persistence model, not only
  storage location.
- OpenDAL itself is `ContinuityStorageBackend`: storage services, routing /
  layering, and the blocking `Operator` handle.
- Do not introduce a second Beluna-owned storage backend trait for this slice.
- The first implementation should use OpenDAL's blocking API to preserve the
  current synchronous Continuity API shape.

Preferred candidate:

```text
ContinuityEngine
  owns ContinuityStore
  uses ContinuityPersistence

ContinuityPersistence
  owns OpenDAL blocking Operator
  owns Continuity object/key layout
  owns store serialization and migration hook
  load_store() -> ContinuityStore
  save_store(store)

ContinuityStorageBackend = OpenDAL configured service/router/layers/operator
```

This intentionally does not reintroduce a bespoke `ContinuityStorageBackend`
trait. OpenDAL provides the backend polymorphism. Continuity still owns the
logical store contract, record body representation, namespace/revision policy,
serialization for the current layout, and migration policy. If a future
physical layout stores records as table rows or binary files, that should be
modeled as an OpenDAL-backed service/layout choice from Continuity's point of
view.

OpenDAL notes:

- Apache OpenDAL provides a Rust `Operator` abstraction over backends including
  filesystem and object stores.
- OpenDAL has a memory service suitable for tests.
- The filesystem service supports read, write, stat, delete, list, create_dir,
  and copy.
- The Rust API supports async operations and also has a blocking API.
- Continuity should not expose OpenDAL types in its domain-level record API.
  OpenDAL is allowed at the persistence/configuration boundary because it is
  the storage backend abstraction.

Open:

- Whether implementation pressure requires a new `ContinuityError::Storage`
  variant, or whether the existing error shape is enough.

Implementation default:

- Continuity promises single-process logical consistency.
- Backend-level atomic write guarantees are not generalized yet.
- Backend errors are normalized into `ContinuityError`.
- The first backend persists a JSON document for simplicity, but the logical
  backend contract uses record bodies as bytes plus content type so future
  binary/table backends are not forced through `serde_json::Value`.

## Migration Hook

Confirmed:

- Do not implement migration now.
- Leave an architectural hook for future migration.

Migration dimensions:

- storage backend migration, e.g. fs JSON to OpenDAL-backed object storage.
- data schema migration, e.g. store envelope version or namespace payload schema
  changes.

Candidate shape:

```text
ContinuityStoreMigrator
  migrate_loaded_store(store, backend_metadata?) -> store
```

Implementation default:

- Use one no-op load-boundary migration hook for now.
- Split backend migration and schema migration only when a real migration is
  introduced.
