# Moira Verification

## Behavioral Checks

1. Moira can register a known local build and use that selected launch target for the next wake through Loom.
2. Moira can create, edit, persist, and reselect multiple local JSONC profile documents through Loom.
3. Moira can wake the selected launch target with the selected profile, or wake with profile omitted and omit `--config`.
4. Atropos exposes runtime status, graceful stop, explicit force-kill with second confirmation, and app-exit stop behavior.
5. Host-native Loom exposes Lachesis, Atropos, and Clotho capabilities through explicit host UI owners.
6. Apple Universal exposes the first minimum native Loom through a Settings-integrated operations panel.

## Current Clotho Follow-On Checks

1. Moira can forge a local launch target from a Beluna repo root or `core/` crate root before wake.
2. Moira can discover a published Core release for the current supported target and verify it against `SHA256SUMS` before activation.
3. Moira can install the verified release into a version-isolated local directory and expose it as a launch target.
4. Wrapper profile documents can package `core_config`, `env_files`, and inline environment variables through Loom.
5. Moira still defers selected JSONC profile validation against the Core schema authority to a later slice.

## Rust Build Checks

1. Local Moira runtime verification passes with `cargo check --manifest-path moira/runtime/Cargo.toml --locked`.
2. Local Moira runtime tests pass with `cargo test --manifest-path moira/runtime/Cargo.toml --locked`.
3. Transitional Tauri adapter verification passes with `cargo check --manifest-path moira/src-tauri/Cargo.toml --locked`.
4. DuckDB bundled verification is explicit through the runtime and adapter checks with `--features duckdb-bundled`.
5. Cargo metadata for the default Moira feature set resolves DuckDB with the prebuilt DuckDB path.
6. Cargo metadata with `--features duckdb-bundled` resolves `duckdb/bundled` and `libduckdb-sys/bundled`.

## Embedded Runtime Checks

1. Moira backend runtime can be constructed from host-provided paths.
2. `MoiraRuntime` reports receiver bind conflicts through resource status.
3. `moira/runtime/tests/runtime_open.rs` covers runtime open, directory creation, receiver readiness, and receiver bind conflict status.
4. `moira/runtime/tests/clotho_prepare.rs` covers Clotho registration and profile-backed wake preparation through the public runtime boundary.
5. `moira/runtime/tests/lachesis_ingest.rs` covers OTLP ingest through a tonic client and verifies run/tick/detail projections.
6. `moira/runtime/tests/atropos_supervision.rs` covers Unix process wake and graceful stop through Atropos.
7. Apple Universal can consume the narrow Moira host API needed for minimum Loom.
8. Process-local resource conflicts surface as runtime status.
9. Body endpoint socket discovery remains usable when Core is already listening from another launch path.

## Observability Checks

1. OTLP logs are ingested and persisted locally through Moira runtime.
2. Loom can show wake-scoped inspection from local Moira state plus locally stored Core OTLP logs.
3. Loom can show a tick timeline from locally stored native OTLP log data using native contract fields as the primary contract.
4. Loom can render one Cortex-handled tick as a raw-first native event timeline using native `traceId` as the machine grouping key.
5. The selected-tick workspace exposes native chronology and raw source-grounded inspection before deeper owner-specific reconstruction.
6. Cortex View recognizes ticks with native Cortex owner evidence while preserving broader tick inspection from Lachesis.
7. Raw inspection surfaces `record_kind` so operators can distinguish native owner records, legacy contract records, and ordinary logs during migration.
8. Legacy contract payloads remain readable through compatibility normalization while native records keep scope/event/trace/span identity.
9. Loom can inspect AI transport, AI Chat, Stem, Spine, and goal-forest data through raw records first and targeted projections as those projections mature.
10. Goal-forest comparison is derived from selected ticks rather than loaded from a precomputed diff artifact.
11. Metrics/traces surfaces show exporter status and handoff links.

## Evidence Homes

1. Moira Clotho preparation and Atropos supervision logic in the Moira backend runtime.
2. Core OTLP event-shape tests, [Core Observability](../core/observability.md), contract fixtures, and config validation guardrails.
3. Live end-to-end operator walkthroughs remain valid evidence for wake/stop and browse-surface checks while the current local read models and control-plane slices continue to stabilize.

## Current Architecture Checks

1. Current wake list, broader tick list, Cortex timeline mode, and raw-event inspection remain operator-equivalent after the Cortex View reshape.
2. Tauri command handlers delegate through `app` into explicit backend owners rather than accumulating ownership directly.
3. Loom root views delegate live refresh wiring and selection-state orchestration; bridge transport delegates normalization and sorting.
4. Bridge contracts, normalized Loom-facing models, and query-owned UI state remain distinct layers rather than collapsing back into one shared frontend type bucket.
5. Lachesis persistence and Lachesis projections remain the owner of Lachesis state; Clotho and Atropos state use dedicated ownership paths.
6. Clotho durable manifests and profile documents remain app-local preparation truth, while current selected launch-target/profile refs remain query-owned session state until an explicit persistence slice lands.
7. Shared shell chrome such as feature tabs and dialog scaffolding remains reusable, with feature-specific preparation, supervision, and observability semantics owned by feature namespaces.
