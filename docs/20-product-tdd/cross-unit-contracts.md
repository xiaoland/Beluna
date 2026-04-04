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
- Endpoint units and Moira may emit local app diagnostics but must not duplicate core runtime observability control surfaces.

6. Local control-plane and observability consumer contract
- `moira` may prepare local Core profiles and artifact selections, but `core` remains the typed config shape authority.
- `moira` may wake, stop, and locally supervise Core processes, but `core` remains the authority for runtime behavior after launch.
- `moira` may ingest and query Core OTLP logs as a local observability consumer/storage surface, but log semantics and export policy remain core-owned.
- Metrics and traces may be surfaced through exporter status and handoff links without making Moira the authority for those signals.

7. Core release packaging contract
- `#8` is the producer-side release workflow that publishes GitHub Release assets for Moira consumption.
- The minimum archive naming contract is `beluna-core-<rust-target-triple>.tar.gz`.
- The minimum checksum contract is a release-level `SHA256SUMS` file covering the published archives.
- The archive may contain an executable named `beluna`; archive basename and embedded binary basename are not required to match exactly.
- The current first consumer contract is locked to `aarch64-apple-darwin` before broader target expansion.

8. Log inspection contract
- Required cross-unit structured observability surfaces and reconstruction guarantees are defined in `docs/20-product-tdd/observability-contract.md`.
- Core-owned event-family naming and Moira-owned Loom composition remain unit-local as long as those guarantees remain intact.

## Compatibility Rule

Cross-unit contract changes require synchronized updates to Product TDD contract definitions and affected Unit TDD interface/operation docs.
