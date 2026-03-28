# Cross-Unit Contracts

System-level contracts between units are authoritative here.

## Contract Catalog

1. Endpoint protocol contract
- Interface: Unix socket NDJSON between `core` and external endpoint units.
- Stability expectation: compatible auth/register/message flow for supported endpoint clients.
- Failure semantics: explicit terminal outcomes and transport-visible failures.

2. Dispatch outcome contract
- Terminal outcomes remain explicit: `Acknowledged`, `Rejected`, `Lost`.
- Endpoint clients must treat non-ack outcomes as first-class runtime results.

3. Identity contract
- Endpoint and signal identity remains explicit, including fully qualified signal IDs.
- Correlated result senses include `act_instance_id` correlation semantics.

4. Configuration contract
- `core` typed config boundary is the shape authority.
- External units consume resulting runtime behavior, not an independent config schema authority.

5. Observability ownership contract
- `core` owns runtime observability export.
- Endpoint units may emit local app diagnostics but must not duplicate core runtime observability control surfaces.

6. Local log consumption contract
- `core` emits local NDJSON runtime logs under configured `logging.dir`.
- `monitor` consumes those files as read-only artifacts and must tolerate malformed lines without redefining core ownership.
- Filtering/search behavior in `monitor` is a consumer concern; log field production remains core-owned.

## Compatibility Rule

Cross-unit contract changes require synchronized updates to Product TDD contract definitions and affected Unit TDD interface/operation docs.
