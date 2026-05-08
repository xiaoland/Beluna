# Moira Interfaces

## External Interface

1. Host runtime API:
- Embedded Moira backend runtime consumed by Beluna Human Interface hosts.
- Rust crate: `moira/runtime`.
- Primary handle: `MoiraRuntime`, opened from host-provided `MoiraRuntimeConfig`.
- Typed Clotho, Lachesis, and Atropos query/operation surfaces.
- Runtime status, resource status, minimum Loom snapshot queries, and event/pulse delivery for host-native Loom UI.
- Apple Universal is the first host for the minimum native Loom surface.
- First Apple binding proof: `moira/ffi` builds `libmoira_ffi.dylib` and exposes `moira_runtime_status_json`, `moira_runtime_loom_json`, `moira_runtime_shutdown_json`, plus string-freeing ABI.
- Apple Universal macOS packaging bundles `libmoira_ffi.dylib` and DuckDB's `libduckdb.dylib` into the host app's `Contents/Frameworks`.

2. Transitional desktop entrypoint:
- Current Tauri/Vue app remains a migration container over `moira/runtime` while Apple coverage lands.

3. Artifact preparation interface:
- GitHub Releases discovery for published Core artifacts.
- Trusted checksum file: `SHA256SUMS`.
- Trusted Core archive pattern: `beluna-core-<rust-target-triple>.tar.gz`.
- Current macOS-first expected asset: `beluna-core-aarch64-apple-darwin.tar.gz`.
- The published archive may contain executable `beluna`; archive basename and embedded executable basename may differ.
- Local source-folder input accepts a Beluna repo root or `core/` crate root for explicit development forge before launch.
- App-local JSONC profile documents managed under Clotho-owned profile ids.

4. Lifecycle supervision interface:
- Wake local Core with a selected Clotho launch target and JSONC profile.
- Graceful stop for supervised Core.
- Explicit force-kill behind second confirmation.

5. Observability interface:
- Local OTLP gRPC logs receiver.
- Raw-event query and live-pulse interfaces for host-native Loom.
- Aggregate minimum Loom snapshot through `MoiraRuntime::loom_snapshot(selection)`, combining runtime status, Clotho launch targets/profiles, Lachesis wake/tick summaries, and selected tick raw detail.
- Minimum guaranteed log-backed Loom surfaces:
  - wake list
  - tick list
  - selected tick workspace
  - raw-first native event timeline anchored by selected tick and native `traceId`
  - raw event inspector as the source-grounded inspection surface, including native/legacy/ordinary `record_kind`
  - Cortex interval inspection when matching boundary records are reconstructable
  - AI transport, AI Chat, Stem, Spine, and goal-forest projections when their native owner events are available
- Metrics/traces exporter-status surfaces and handoff links only.

## Consumed Contract

1. Core typed config boundary remains the schema authority.
2. Core OTLP logs satisfy the cross-unit reconstruction rules defined in `docs/20-product-tdd/observability-contract.md` and the current owner scope / `eventName` surface described in `docs/30-unit-tdd/core/observability.md`.
3. Core startup/shutdown semantics remain Core-owned even when Moira supervises the process locally.
4. Legacy Core contract logs remain readable through Lachesis compatibility normalization during the migration period.

## Embedding Constraint

1. The host API exposes Moira-owned behavior through typed runtime boundaries.
2. Hosts provide explicit Moira paths, receiver bind address, event sink, and task spawner.
3. Apple Universal first slice uses a process-local embedded Moira runtime.
4. Cross-client Owner/Attach authority coordination belongs to later design.
5. Host-native Loom UI may choose its own layout while preserving Moira-owned query/control semantics.
6. The C ABI proof returns JSON for the first status and minimum Loom snapshot slices; broader Loom APIs should move toward typed binding ownership as the surface grows.
