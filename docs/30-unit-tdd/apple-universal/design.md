# Apple Universal Design

## Responsibility

1. Provide chat-oriented Human Interface UX on Apple platforms.
2. Manage connection lifecycle to Core Unix socket endpoint.
3. Persist and restore local endpoint-side chat history.
4. Host native Moira operator surfaces through Apple-native panels.
5. Present Moira runtime status, receiver status, Core control context, and raw-first local observability browsing.

## Boundary Rules

1. Core retains runtime behavior, endpoint protocol authority, and observability emission semantics.
2. Moira retains local preparation, supervision, observability ingestion/storage/query/projection, and future platform adapter semantics.
3. Apple Universal owns Apple-native presentation, interaction flow, endpoint socket discovery, and app-local UI state.
4. Keep protocol compatibility explicit and typed.
5. Keep socket I/O, Moira runtime calls, and decoding off the main thread to preserve UI responsiveness.
6. Allow multiple Apple Universal app instances. Core/Spine assigns runtime body endpoint ids for each authenticated socket session.

## Moira Loom Shape

The issue #30 minimum Apple-native Moira Loom proof lands as a Settings-integrated operations panel.

This surface is judged against the issue #30 minimum Apple contract. Legacy Tauri/Vue Loom behavior is migration evidence, and Apple Universal selects native interaction shape independently.

Initial proof sections:

1. Connection:
- selected socket path
- discovered socket candidates
- connect, disconnect, retry
- body endpoint state

2. Core Control:
- selected launch target
- selected profile
- wake action
- stop action when Atropos owns the supervised process
- current Atropos phase

3. Moira Status:
- embedded runtime status
- Lachesis receiver state
- resource conflict banners
- raw event, wake, and tick counts

4. Local Observability:
- wake list
- tick list for selected wake
- selected tick raw records

Follow-on Apple UI splits the minimum proof into three stable panels:

1. Core Control:
- standalone panel parallel to Settings
- Clotho launch-target/profile selection for wake
- Atropos wake, graceful stop, force-kill, terminal reason, and process-state operation UI
- resource conflict and readiness state needed for safe Core lifecycle operations

2. O11y / Lachesis:
- standalone observability and investigation panel parallel to Settings
- wake and tick browsing
- selected tick raw-first inspection
- raw event inspector
- Cortex timeline, narrative investigation, and owner-specific drilldown after each projection has an explicit contract

3. Settings:
- Moira runtime path configuration
- receiver bind address
- default socket candidates
- refresh and diagnostics policy
- host-local preferences for Moira UI behavior

Current Core Control follow-on state:

1. Apple Universal exposes a standalone `Core Control` window parallel to Settings.
2. The panel reuses Moira's host binding state for launch-target/profile selection and runtime status.
3. Wake, graceful stop, and force-kill actions flow through Moira-owned Atropos lifecycle semantics.
4. Force-kill requires a second confirmation path in the host UI.
5. Settings retains Moira status and minimum local observability until the dedicated O11y / Lachesis panel lands.

## Source Boundary Direction

Apple Universal source cleanup is part of the Moira integration path.

1. Settings composition should keep configuration sections separate from operation and investigation panels.
2. Endpoint connection settings, message buffer/history behavior, and Moira operations state should have small explicit owners.
3. Body endpoint wire protocol stays under `BodyEndpoint`.
4. Moira DTOs and runtime calls belong under a Moira-owned app namespace.
5. `ContentView.swift` is a placeholder cleanup candidate because app entry currently uses `ChatView`.
6. App process singleton guards are outside the Apple Universal boundary. Runtime resource conflicts belong to Core or Moira-owned coordination surfaces.

## Current Moira Host Boundary

1. Apple Universal owns a Swift-side Moira namespace under `BelunaApp/Moira`.
2. `MoiraRuntimeClient` is the app-facing binding protocol for Moira runtime snapshots.
3. `MoiraOperationsViewModel` owns the shared minimum Loom state used by Settings status/O11y and the first Core Control panel, keeping refresh and lifecycle work async.
4. `SettingView` receives Moira state explicitly; `ChatViewModel` remains focused on endpoint chat workflows.
5. `MoiraCoreControlPanel` owns Apple-native lifecycle presentation for launch context, Atropos phase, terminal state, wake, stop, and force-kill.
6. The current macOS default client attempts to load `libmoira_ffi.dylib`, calls the C ABI status, minimum Loom snapshot, and lifecycle operation proofs, and maps Rust JSON into Swift DTOs.
7. The first Loom payload uses `MoiraLoomSnapshot` for runtime status, launch targets, profiles, wakes, ticks, and selected tick raw records.
8. The default client reports an unavailable snapshot when the local Rust dylib is absent, keeping Apple-native Moira surfaces usable during packaging work.
9. Follow-on O11y / Lachesis panels may reuse the same binding namespace while owning separate view models and navigation state.
