# Apple Universal Design

## Responsibility

1. Provide chat-oriented Human Interface UX on Apple platforms.
2. Manage connection lifecycle to Core Unix socket endpoint.
3. Persist and restore local endpoint-side chat history.
4. Host the first minimum native Moira Loom surface through a Settings-integrated operations panel.
5. Present Moira runtime status, receiver status, basic Core control context, and raw-first local observability browsing.

## Boundary Rules

1. Core retains runtime behavior, endpoint protocol authority, and observability emission semantics.
2. Moira retains local preparation, supervision, observability ingestion/storage/query/projection, and future platform adapter semantics.
3. Apple Universal owns Apple-native presentation, interaction flow, endpoint socket discovery, and app-local UI state.
4. Keep protocol compatibility explicit and typed.
5. Keep socket I/O, Moira runtime calls, and decoding off the main thread to preserve UI responsiveness.
6. Allow multiple Apple Universal app instances. Core/Spine assigns runtime body endpoint ids for each authenticated socket session.

## First Moira Loom Shape

The first Apple-native Moira Loom lands as a Settings-integrated operations panel.

Initial sections:

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

## Source Boundary Direction

Apple Universal source cleanup is part of the Moira integration path.

1. Settings composition should move toward dedicated section views.
2. Endpoint connection settings, message buffer/history behavior, and Moira operations state should have small explicit owners.
3. Body endpoint wire protocol stays under `BodyEndpoint`.
4. Moira DTOs and runtime calls belong under a Moira-owned app namespace.
5. `ContentView.swift` is a placeholder cleanup candidate because app entry currently uses `ChatView`.
6. App process singleton guards are outside the Apple Universal boundary. Runtime resource conflicts belong to Core or Moira-owned coordination surfaces.

## Current Moira Host Boundary

1. Apple Universal owns a Swift-side Moira namespace under `BelunaApp/Moira`.
2. `MoiraRuntimeClient` is the app-facing binding protocol for Moira runtime snapshots.
3. `MoiraOperationsViewModel` owns Settings-integrated Moira status state and keeps refresh work async.
4. `SettingView` receives Moira state explicitly instead of routing Moira operations through `ChatViewModel`.
5. The current macOS default client attempts to load `libmoira_ffi.dylib`, calls the C ABI status proof, and maps Rust JSON into `MoiraRuntimeSnapshot`.
6. The default client reports an unavailable snapshot when the local Rust dylib is absent, keeping Settings usable during packaging work.
