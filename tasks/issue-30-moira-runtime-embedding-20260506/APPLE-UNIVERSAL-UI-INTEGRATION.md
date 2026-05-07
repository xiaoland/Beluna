# Apple Universal UI Integration

This file captures design notes for integrating the minimum Moira Loom into Apple Universal while preserving body endpoint use.

## Product Shape For This Task

Apple Universal has two related jobs:

1. Human endpoint surface
- Connect to an existing Core body endpoint socket.
- Send and receive chat-oriented body endpoint messages.
- Keep endpoint connection state visible and recoverable.

2. Minimum Moira Loom
- Prepare and supervise Core through embedded Clotho and Atropos where available.
- Observe embedded Lachesis receiver state.
- Browse local wake and tick observability state.
- Inspect selected tick records through raw-first detail.

The user can use Apple Universal as a body endpoint when Core was started by another process or by a previous session.

## Body Endpoint Socket Discovery

Apple Universal should keep socket connection as a first-class path.

Candidate discovery inputs:

- user-configured socket path
- last successful socket path
- app-local runtime path selected by Apple Universal
- platform candidate paths such as `/var/run/beluna.sock` when supported by deployment docs
- paths reported by embedded Moira runtime after Atropos starts Core

The actual default socket contract belongs in Core/deployment docs. UI can present candidates as discoverable connection choices.

## Moira Loom Placement Options

1. Settings-integrated panel
- Fit for first slice.
- Co-locates socket path, Core status, and Moira status.
- Keeps the app compact.

2. Dedicated Loom tab
- Fit when wake/tick browsing becomes a frequent workflow.
- Gives space for split navigation and raw event detail.

3. macOS secondary window
- Fit for richer desktop operator workflows.
- Can evolve after the minimum slice.

## Recommended First Layout

Use a settings-integrated operations panel for the first implementation.

Sections:

- Connection
  - selected socket path
  - discovered candidates
  - connect / disconnect / retry
  - body endpoint state

- Core Control
  - selected launch target
  - selected profile
  - wake action
  - stop action when Atropos owns the process
  - current Atropos phase

- Moira Status
  - embedded runtime status
  - Lachesis receiver state
  - resource conflict banners
  - raw event / wake / tick counts

- Local Observability
  - wake list
  - tick list for selected wake
  - selected tick raw records

## Interaction Principles

- Socket connection stays usable as the direct body endpoint path.
- Moira controls explain what they own in the current process.
- Resource conflicts appear as actionable status.
- Raw-first inspection remains the reliable minimum observability surface.
- Apple-native controls and navigation shape the UI.

## Source Cleanup Dependency

The first UI slice should clean Apple Universal boundaries before adding Moira sections.

Cleanup goals:

- Settings owns layout composition through dedicated section views.
- Endpoint connection state and socket path editing have a small owner.
- Message buffer/history behavior has a small owner.
- Moira operations state enters as a separate owner.
- Body endpoint wire protocol stays inside `BodyEndpoint`.

Current pressure points:

- `ChatViewModel.swift` is large and owns endpoint lifecycle, retry policy, message paging, local persistence, action handling, and settings draft state.
- `SettingView.swift` already mixes connection controls, chat retention controls, and status.
- `ChatView.swift` relies on multiple computed view helpers and inline actions.
- `ContentView.swift` is still the template placeholder.

## Open UI Decisions

1. Should wake/stop controls live next to socket connection controls or in a separate Core Control section?
2. Should wake/tick browsing appear in Settings for the first slice or move immediately into a dedicated tab?
3. Should raw event detail start as disclosure groups or a table-plus-detail view on macOS?
4. Should iOS/iPadOS show the same surface or gate the minimum Loom to macOS first?
5. How should the UI label a conflict where another process already owns the OTLP receiver?
