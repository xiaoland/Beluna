# AGENTS.md of Beluna Apple Universal App

Beluna Apple Universal App is the app that bridges human interaction with Beluna Core in Apple ecosystem.

## Tech Stacks

- Language: Swift
- Platform: macOS (for now), iOS, iPadOS
- UI: SwiftUI (use AppKit bridge only when necessary)

## Current Focus

- Harden desktop endpoint UX and connection lifecycle reliability for Beluna Core integration.

## Boundary and Quality Rules

- Keep desktop logic as a Body Endpoint of Core; do not re-implement Core domain logic in desktop.
- Treat socket protocol compatibility as a contract; prefer explicit typed request/response mapping.
- Keep UI responsive: socket I/O and parsing must not block the main thread.
- Add tests for protocol decoding/encoding and connection lifecycle behavior where practical.

## Design Sources

- Product intent and invariants: `../docs/10-prd/index.md`
- System and unit boundaries: `../docs/20-product-tdd/index.md`, `../docs/30-unit-tdd/index.md`
- Deployment and operations constraints: `../docs/40-deployment/index.md`

## Current State

> Last Updated At: 2026-03-12T17:10+08:00

### Live Capabilities

- SwiftUI desktop chat endpoint connects to Core via Unix Socket NDJSON (`UnixSocketBodyEndpointClient`).
- Connection controls are exposed in `SettingView` (socket path, connect/disconnect, retry).
- Chat history controls are exposed in `SettingView` (message buffer capacity + local Sense/Act history clearing).
- Connection intent and socket path are persisted via `UserDefaults`.
- Local Sense/Act chat traffic is persisted to disk and restored on app relaunch.
- App enforces single-instance runtime lock on macOS.
- Xcode debug sessions default to manual connect.
- Socket path is configured directly from `SettingView` and applied immediately on reconnect.
- Chat view keeps a bounded in-memory message ring buffer and incrementally loads older/newer pages on scroll.
- Beluna lifecycle state uses `Hibernate` (instead of `Sleeping`) when Core is unavailable after connection history exists.
- Auth `ns_descriptors` follow Apple endpoint identity and semantic IDs:
- endpoint IDs: `apple.universal` / `macos.app` / `ios.app`
- act: `present.message.text`
- senses: `user.message.text`, `present.message.text.success`, `present.message.text.failure`
- sense payload schemas are intentionally simple and text-only.
- correlated result senses carry `act_instance_id` as sense body field.
- Spine may canonicalize endpoint id to generated body endpoint id on auth.

### Known Limitations & Mocks

- In-chat observability surfaces are intentionally removed from Apple Universal; runtime metrics/logs are handled by Core-side observability.
- Local history persistence currently focuses on Sense/Act traffic only; runtime system/debug notices are intentionally not replayed.
- Protocol/lifecycle tests should be expanded for reconnect edge cases and large-history pagination.

### Immediate Next Focus

- Add test coverage for connection lifecycle and pagination behaviors.
- Improve in-chat filtering/search for large local Sense/Act history.
