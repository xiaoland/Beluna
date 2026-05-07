# Moira Operations

## Startup

1. A host app creates `MoiraRuntime` with explicit local paths, receiver bind address, event sink, task spawner, and later platform adapter selection.
2. The runtime loads local Clotho preparation state and Atropos supervision state.
3. The runtime initializes the local OTLP logs receiver and storage backend.
4. The host exposes Loom UI after Moira runtime status and resource status are available.

## Wake Flow

1. Host UI state provides the current selected Clotho launch-target ref plus the optional selected profile id for the next wake.
2. Clotho resolves the selected launch target, derives the optional profile path from `profile_id`, and returns prepared wake input to Atropos.
3. Atropos ensures the OTLP logs receiver is ready before starting supervised Core.
4. Atropos launches Core and records supervised wake tracking.
5. Host-native Loom refreshes runtime status through Atropos query orchestration while Lachesis reacts to ingest updates for wake and tick browsing.
6. Schema validation against Core authority remains deferred to a later slice.

## Clotho Preparation Flow

1. Registering a known local build writes or updates a durable manifest under Clotho ownership.
2. Explicit forge resolves a Beluna repo root or `core/` crate root, runs a development build, and writes or updates the resulting known local build manifest.
3. Published release discovery reads the current GitHub Release catalog for the supported artifact contract.
4. Installing a published release downloads the target archive plus `SHA256SUMS`, verifies the checksum, extracts the executable into a version-isolated install directory, and writes the installed artifact manifest.
5. Host-native Loom may browse launch targets. The current selected launch target remains host session state until a later persistence slice lands.

## Rust Build Storage Flow

1. Local Rust builds share the repo-root Cargo target directory configured by `.cargo/config.toml`.
2. Local Moira backend builds use the prebuilt DuckDB library path through `DUCKDB_DOWNLOAD_LIB=1`.
3. macOS debug Moira binaries embed `@loader_path/deps` as a runtime search path so the prebuilt DuckDB dylib in `target/debug/deps` is visible to VSCode Launch and direct binary startup.
4. Routine runtime verification runs `cargo check --manifest-path moira/runtime/Cargo.toml --locked` and `cargo test --manifest-path moira/runtime/Cargo.toml --locked`.
5. Routine Tauri adapter verification runs `cargo check --manifest-path moira/src-tauri/Cargo.toml --locked` from the repo root.
6. Source-bundled DuckDB verification runs the same runtime and adapter checks with `--features duckdb-bundled`.
7. Local Rust storage preview runs `bash scripts/rust-storage-maintenance.sh sweep-all-dry-run`.
8. Local Rust storage cleanup runs `bash scripts/rust-storage-maintenance.sh sweep-all`.

## Embedded Runtime Flow

1. Apple Universal embeds `moira/runtime` as the first Human Interface host.
2. The first Apple implementation uses process-local `MoiraRuntime`.
3. Resource conflicts surface as runtime status.
4. Body endpoint socket discovery remains available for Core processes started by another process or prior session.
5. Future Owner/Attach coordination can promote one local Moira authority per user/session or configured scope.

## Observability Flow

1. Receive Core OTLP logs locally.
2. Persist raw events before updating any derived read model.
3. Project the baseline read models needed for Loom wake list and tick timeline.
4. Project any additional chronology, interval-pairing, or targeted lookup indexes that materially improve operator-facing browsing, with raw storage as source of truth.
5. Resolve the selected tick through Cortex View first, using timeline or narrative mode for handled ticks, then sectional Stem / Spine investigation, and finally source-grounded raw-event inspection.
6. Surface metrics/traces exporter status and handoff links.

## Shutdown

1. When a host that owns a supervised Core exits, the host asks Atropos for graceful stop according to host lifecycle policy.
2. Offer explicit force-kill only through a second confirmation path.
3. Flush local observability state and close control-plane resources.

## Failure Handling

1. Checksum mismatch blocks published artifact activation.
2. Missing target asset or broken archive blocks published artifact activation.
3. Local source build failure blocks wake and surfaces explicit failure state.
4. OTLP receiver/storage readiness failure blocks supervised wake.
5. Unexpected Core exit becomes explicit terminal supervision state visible in Loom.
6. Missing selected launch target input, or an explicitly selected but unresolved profile input, blocks wake rather than letting Atropos invent a fallback path.

## Current Extension Boundary

1. Extend explicit owners through runtime modules and host UI sections.
2. New preparation flows land under `clotho` backend ownership plus host-native Clotho UI owners.
3. New supervision flows land under `atropos` backend ownership plus host-native Atropos UI owners.
4. Shared shell affordances belong to the host UI, while feature-specific semantics remain inside the corresponding mythic namespace.
5. Future persistence must choose an explicit owner before choosing filesystem, database, or app-state storage shape.
