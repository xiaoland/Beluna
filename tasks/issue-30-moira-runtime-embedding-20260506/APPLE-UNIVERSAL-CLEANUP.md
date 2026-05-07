# Apple Universal Source Cleanup

This file captures cleanup scope for Apple Universal before the minimum Moira Loom enters the Settings-integrated panel.

## Why This Belongs In The Task

The accepted first UI slice puts Moira Loom into Settings. The current Apple Universal implementation already mixes several responsibilities in the same view and view model. Adding Moira directly there would make later extraction harder.

## Current Observations

1. `ChatViewModel.swift`
- Around 727 lines.
- Owns socket path defaults, connection enablement, reconnect state, message draft, message paging, local persistence, inbound act handling, system messages, and settings drafts.
- Contains business logic that will compete with Moira runtime state once operations UI arrives.

2. `SettingView.swift`
- Owns Connection, Chat, and Status presentation in one view.
- Accepted as the first Moira Loom container.
- Needs dedicated section subviews before adding Moira Status and Local Observability.

3. `ChatView.swift`
- Around 269 lines.
- Uses computed view helpers for header, hibernation notice, message list, composer, pills, and pagination hints.
- Inline button actions and lifecycle calls can be made thinner while preserving behavior.

4. `BodyEndpoint`
- Protocol and socket code is already separated into `BodyEndpoint`.
- `UnixSocketBodyEndpointClient.swift` is sizable and should remain protocol/transport owned.
- Socket discovery can sit beside endpoint connection settings while the wire protocol stays here.

5. `ContentView.swift`
- Still contains the SwiftUI template placeholder.
- The app entry uses `ChatView`, so this file is a cleanup candidate.

## Target Ownership

Suggested first-slice shape:

```text
apple-universal/BelunaApp/
  App/
    BelunaAppApp.swift
    AppRuntimeEnvironment.swift
    AppRuntimeGuard.swift
    Chat/
      ChatView.swift
      ChatViewModel.swift
      ChatMessage.swift
      MessageBuffer.swift
      LocalSenseActHistoryStore.swift
    Settings/
      SettingView.swift
      ConnectionSettingsSection.swift
      ChatRetentionSettingsSection.swift
      RuntimeStatusSection.swift
      MoiraOperationsSection.swift
      LocalObservabilitySection.swift
    BodyEndpoint/
      ...
    Moira/
      MoiraRuntimeClient.swift
      MoiraStatusModels.swift
      MoiraOperationsStore.swift
```

The exact file layout can adapt to Xcode project ergonomics. The important boundary is ownership clarity.

## Cleanup Rules

- Prefer dedicated SwiftUI section views over growing `SettingView`.
- Keep view bodies mostly declarative.
- Move non-trivial button actions into small methods or service calls.
- Keep socket protocol encoding and decoding under `BodyEndpoint`.
- Keep Moira DTOs and runtime calls under a Moira-owned app namespace.
- Keep local chat history separate from Moira telemetry storage.

## First Cleanup Slice

Recommended before Moira UI:

1. Split Settings into section views.
2. Extract message buffer/page state from `ChatViewModel`.
3. Extract socket path defaults and candidate discovery from `ChatViewModel`.
4. Remove `ContentView.swift` from the target if the Xcode project references allow it.
5. Add a placeholder `MoiraOperationsSection` fed by static preview data or a tiny local model.

## Verification

- Xcode build passes.
- Existing chat connect/send/retry behavior works.
- Existing settings apply socket path and message capacity.
- Existing local history clear/persist behavior works.
- Unit tests cover extracted message buffer and socket candidate logic where practical.

## Open Cleanup Questions

1. Should `ChatViewModel` remain the root app view model for the first slice, or should Settings get a separate store?
2. Should `SettingView` receive smaller stores dedicated to each settings area?
3. Should `ContentView.swift` be deleted immediately or left until Xcode project cleanup confirms target references?
4. Should socket discovery candidates live in `BodyEndpoint`, `Settings`, or a small `RuntimeDiscovery` owner?
