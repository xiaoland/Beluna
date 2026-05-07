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

## Slice 2: Runtime API Boundary

Goal: introduce a host-independent Moira runtime API sized for the Apple Universal minimum Loom and process-local resource status.

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

- Backend compiles with service code reachable through host-independent API.
- Tauri-specific types live in adapter modules during the transition window.
- Unit tests can instantiate runtime services through direct Rust test harnesses.
- Unit tests cover resource-claim success and conflict reporting.

## Slice 3: Moira Runtime Extraction And Tauri Removal Prep

Goal: extract reusable Moira runtime code from the current Tauri app container and prepare full Tauri/Vue removal after Apple coverage.

Targets:

- Replace `tauri::AppHandle` event emission with `MoiraEventSink`.
- Replace `tauri::async_runtime::spawn` with an injected task spawner or Tokio-owned runtime service.
- Keep command facades thin during the transition.

Verification:

- Existing Moira backend checks pass.
- Existing frontend behavior remains reachable through the transitional adapter while replacement hosts are prepared.
- Tauri deletion candidates are listed with Apple replacement coverage.

## Slice 4: Apple Universal Host Integration

Goal: embed Moira backend into Apple Universal as an internal package and expose the first native minimum Loom surface through process-local runtime calls.

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

## Slice 5: Apple Universal Minimum Loom

Goal: implement the minimum Apple-native Loom workflow for this task.

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

## Slice 6: Tauri/Vue Loom Retirement Gate

Goal: decide the Tauri/Vue Loom retirement scope for this issue or a follow-on issue.

Gate criteria:

- Apple Universal covers the minimum operator workflows selected for this issue.
- Durable docs refer to Apple Universal-hosted minimum Loom.
- Remaining Tauri/Vue features have an explicit follow-on owner.

Verification:

- Workspace builds and tests pass with the chosen retirement scope.
- Packaging scripts and maintenance scripts match the chosen scope.

## Future Slices

These are design context for this packet and should become separate task packets before implementation:

- CLI Moira host commands.
- Windows native host.
- Full Apple-native Loom.
- Sandbox platform adapters.
- Ledger platform adapters.
