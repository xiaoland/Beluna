# Sequence

## Slice 0: Durable Restatement

Goal: align architecture docs before code movement.

Likely files:

- `docs/20-product-tdd/unit-topology.md`
- `docs/20-product-tdd/unit-to-container-mapping.md`
- `docs/20-product-tdd/cross-unit-contracts.md`
- `docs/30-unit-tdd/moira/*`
- `docs/30-unit-tdd/apple-universal/*`
- `moira/AGENTS.md`
- `apple-universal/AGENTS.md`

Verification:

- Docs clearly distinguish Moira runtime unit, Human Interface clients, platform adapters, and retired Tauri/Vue Loom surface.
- Docs keep Core authority boundaries intact.

## Slice 1: Apple Universal Source Cleanup

Goal: prepare Apple Universal source boundaries for the Settings-integrated Moira panel.

Targets:

- Split `SettingView` into section-level SwiftUI subviews.
- Reduce `ChatViewModel` responsibility concentration by extracting endpoint connection settings, message buffer/history behavior, and socket discovery candidates where practical.
- Keep body endpoint protocol logic inside `BodyEndpoint`.
- Keep app runtime guards separate from Moira runtime concerns.
- Remove or retire placeholder entry files that mismatch the app surface.

Verification:

- Apple Universal builds.
- Existing chat and settings behavior stays intact.
- Focused tests cover any extracted non-UI logic.
- The Settings surface has an explicit insertion point for Moira Status and Local Observability sections.

## Slice 2A: Runtime API Boundary Packet

Goal: define the host-independent Moira runtime API sized for the Apple Universal minimum Loom and process-local resource status before moving Rust code.

Candidate shape:

- `MoiraRuntime`
- `MoiraRuntimeConfig`
- `MoiraPaths`
- `MoiraEvent`
- `MoiraEventSink`
- `MoiraTaskSpawner`
- `MoiraResourceStatus`
- `MoiraResourceConflict`
- typed modules for Clotho, Lachesis, and Atropos commands/queries

Verification:

- Task packet records the current seams, DTO shape, and extraction map.
- Runtime API keeps Core authority boundaries intact.
- Slice 2B can start from a bounded file map and verification gate.

Primary artifacts:

- `RUNTIME-API-BOUNDARY.md`
- `RUNTIME-API-DTO-SKETCH.md`
- `RUNTIME-API-EXTRACTION-MAP.md`

## Slice 2B: Runtime API Implementation

Goal: introduce the host-independent Rust runtime API and keep the legacy desktop shell as a thin adapter during the transition.

Status: implemented locally.

Candidate shape:

- `moira/runtime` workspace crate
- `MoiraRuntime`
- `MoiraRuntimeConfig`
- `MoiraPaths`
- `MoiraEvent`
- `MoiraEventSink`
- `MoiraTaskSpawner`
- typed facades for Clotho, Lachesis, and Atropos commands/queries

Verification:

- Backend compiles with service code reachable through host-independent API.
- Desktop-shell-specific types live in adapter modules during the transition window.
- Unit tests can instantiate runtime services through direct Rust test harnesses.
- Unit tests cover resource-claim success and conflict reporting.

Implemented verification:

- `cargo check --manifest-path moira/runtime/Cargo.toml --locked`
- `cargo test --manifest-path moira/runtime/Cargo.toml --locked`
- Transitional desktop adapter compiled during this slice before retirement.

## Slice 2C: Runtime Integration Tests

Goal: add critical public-boundary tests for `moira/runtime` before Apple binding work.

Status: implemented locally.

Coverage:

- runtime open, directory creation, receiver readiness, and resource status
- receiver bind conflict as `MoiraResourceState::Conflict`
- Clotho known local build registration plus profile-backed wake preparation
- Lachesis OTLP ingest through a tonic client, including run/tick/detail projections
- Atropos wake and graceful stop of a Unix process fixture

Implemented verification:

- `cargo test --manifest-path moira/runtime/Cargo.toml --locked`
- Transitional desktop adapter compiled during this slice before retirement.

## Slice 3: Legacy Desktop Removal Prep

Goal: prepare full legacy desktop removal after Apple coverage by shrinking the remaining transitional adapter and frontend dependency surface.

Targets:

- Keep command facades thin during the transition.
- Keep event/task handling inside adapter code.
- List remaining legacy desktop deletion candidates with Apple replacement coverage.

Verification:

- Existing Moira backend checks pass.
- Existing frontend behavior remains reachable through the transitional adapter while replacement hosts are prepared.
- Legacy desktop deletion candidates are listed with Apple replacement coverage.

## Slice 4: Apple Universal Host Integration

Goal: embed Moira backend into Apple Universal as an internal package and expose the first native minimum Loom surface through process-local runtime calls.

Status: Swift host binding seam, first Rust C ABI status adapter proof, and macOS Xcode packaging automation implemented locally.

Candidate options:

- Swift calls Moira runtime through an explicit internal package binding.
- `apple-universal` receives Moira runtime status and Lachesis receiver status.
- Apple host displays resource conflict status when another process owns a local Moira resource.
- Apple host preserves body endpoint socket discovery and connection.
- UI exposes minimum operator navigation through a SwiftUI-native shape.

Verification:

- Xcode build passes for the host spike.
- The app displays or logs one Moira runtime query result.
- Multi-process smoke test proves the second process reports resource conflicts cleanly.
- Socket discovery smoke test proves body endpoint use when Core is already listening.
- Main-thread responsiveness remains protected.

Implemented verification:

- `xcodebuild test -project apple-universal/BelunaApp.xcodeproj -scheme BelunaApp -destination 'platform=macOS' -only-testing:BelunaAppTests`
- `cargo check -p moira-ffi --locked`
- `cargo test -p moira-ffi --locked`
- `cargo build -p moira-ffi --lib --locked`
- `bash -n apple-universal/script/build_moira_ffi.sh`
- `plutil -lint apple-universal/BelunaApp.xcodeproj/project.pbxproj`
- `xcodebuild build -project apple-universal/BelunaApp.xcodeproj -scheme BelunaApp -destination 'platform=macOS'`
- `codesign --verify --deep --strict /Users/lanzhijiang/Library/Developer/Xcode/DerivedData/BelunaApp-hbfvzmxvgxyigodcpjjrlfecfmtn/Build/Products/Debug/BelunaApp.app`
- `otool -D /Users/lanzhijiang/Library/Developer/Xcode/DerivedData/BelunaApp-hbfvzmxvgxyigodcpjjrlfecfmtn/Build/Products/Debug/BelunaApp.app/Contents/Frameworks/libmoira_ffi.dylib`
- `otool -L /Users/lanzhijiang/Library/Developer/Xcode/DerivedData/BelunaApp-hbfvzmxvgxyigodcpjjrlfecfmtn/Build/Products/Debug/BelunaApp.app/Contents/Frameworks/libmoira_ffi.dylib`
- `xcodebuild test -project apple-universal/BelunaApp.xcodeproj -scheme BelunaApp -destination 'platform=macOS' -only-testing:BelunaAppTests/BelunaAppTests`
- `xcodebuild test -project apple-universal/BelunaApp.xcodeproj -scheme BelunaApp -destination 'platform=macOS' -only-testing:BelunaAppTests/MoiraRuntimeBindingTests`
- `xcodebuild test -project apple-universal/BelunaApp.xcodeproj -scheme BelunaApp -destination 'platform=macOS' -only-testing:BelunaAppTests`

Packaging proof:

- `Build Moira FFI` invokes Cargo from the BelunaApp macOS target.
- `libmoira_ffi.dylib` and `libduckdb.dylib` are copied into `BelunaApp.app/Contents/Frameworks` and signed.
- `dynamicClientLoadsBundledMoiraFFI` loads the bundled FFI dylib and calls real Moira runtime status.

## Slice 5: Apple Universal Minimum Loom

Goal: implement the minimum Apple-native Loom workflow for this task.

Status: implemented locally for read/query surfaces.

Minimum surface:

- Runtime/receiver status.
- Launch-target/profile read surface sized to support wake context.
- Wake list.
- Tick list.
- Selected tick raw-first inspection.

Verification:

- Apple Universal can browse Moira-owned local observability state.
- The UI is SwiftUI-native and keeps socket I/O plus Moira calls off the main thread.
- Focused tests cover binding DTO decoding and view-model state transitions where practical.

Implemented verification:

- `cargo check -p moira-ffi --locked`
- `cargo test -p moira-ffi --locked`
- `cargo test --manifest-path moira/runtime/Cargo.toml --locked`
- `bash -n apple-universal/script/build_moira_ffi.sh`
- `plutil -lint apple-universal/BelunaApp.xcodeproj/project.pbxproj`
- `xcodebuild test -project apple-universal/BelunaApp.xcodeproj -scheme BelunaApp -destination 'platform=macOS' -only-testing:BelunaAppTests/MoiraRuntimeBindingTests`

Implementation proof:

- `MoiraRuntime::loom_snapshot(selection)` aggregates runtime status, Clotho launch targets/profiles, Lachesis wake/tick summaries, and selected tick raw detail.
- `moira_runtime_loom_json` exposes that snapshot through the first Apple FFI adapter.
- Apple Universal decodes `MoiraLoomSnapshot`, keeps selection state in `MoiraOperationsViewModel`, and renders the first Settings-integrated Loom read/query panel.

## Slice 6: Tauri/Vue Loom Retirement Gate

Goal: decide the Tauri/Vue Loom retirement scope for this issue or a follow-on issue.

Status: retirement gate and deletion implemented locally.

Gate criteria:

- Apple Universal covers the minimum operator workflows selected for this issue.
- Durable docs refer to Apple Universal-hosted minimum Loom.
- Remaining Tauri/Vue features have an explicit follow-on owner.
- Acceptance follows the issue #30 Apple Universal minimum Loom contract.

Verification:

- Workspace builds and tests pass with the chosen retirement scope.
- Packaging scripts and maintenance scripts match the chosen scope.

Decision:

- The gate is ready because Slice 5 covers the selected minimum Apple Universal Loom contract.
- Legacy Clotho mutation, Atropos operation, and richer Lachesis inspection ideas move to follow-on packets.
- Tauri/Vue deletion proceeded in this slice with the guardrails in `TAURI-LOOM-RETIREMENT.md`.

Implemented deletion scope:

- Removed the legacy Vue frontend and Tauri desktop container.
- Removed frontend package, Vite, TypeScript, and Tauri build metadata.
- Removed the Tauri workspace member from root Cargo metadata.
- Updated active maintenance scripts and durable docs to use `moira/runtime` and `moira/ffi` as the Moira code surface.

## Future Slices

These are design context for this packet and should become separate task packets before implementation:

- CLI Moira host commands.
- Windows native host.
- Full Apple-native Loom.
- Sandbox platform adapters.
- Ledger platform adapters.
