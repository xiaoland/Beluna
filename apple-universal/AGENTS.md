# AGENTS.md of Beluna Apple Universal App

Beluna Apple Universal App is the app that bridges human interaction with Beluna Core in Apple ecosystem.

## Tech Stacks

- Language: Swift
- Platform: macOS (for now), iOS, iPadOS
- UI: SwiftUI (use AppKit bridge only when necessary)

## Current Focus

- Harden desktop endpoint UX and in-chat observability for Beluna Core integration.

## Boundary and Quality Rules

- Keep desktop logic as a Body Endpoint of Core; do not re-implement Core domain logic in desktop.
- Treat socket protocol compatibility as a contract; prefer explicit typed request/response mapping.
- Keep UI responsive: socket I/O and parsing must not block the main thread.
- Add tests for protocol decoding/encoding and connection lifecycle behavior where practical.

## Design Sources

- Product overview: `../docs/overview.md`
- Shared glossary: `../docs/glossary.md`
- Contracts and protocol context: `../docs/contracts/README.md`

## Current State

> Last Updated At: 2026-02-22T18:20+08:00

### Live Capabilities

- SwiftUI desktop chat endpoint connects to Core via Unix Socket NDJSON (`SpineUnixSocketBodyEndpoint`).
- Connection controls are exposed in `SettingView` (socket path, connect/disconnect, retry).
- Observability controls are exposed in `SettingView` (metrics endpoint + log directory).
- Chat history controls are exposed in `SettingView` (message buffer capacity).
- Connection intent and socket path are persisted via `UserDefaults`.
- App enforces single-instance runtime lock on macOS.
- Xcode debug sessions default to manual connect.
- Socket path is configured directly from `SettingView` and applied immediately on reconnect.
- Metrics are rendered in Chat header (`cortex_cycle_id`, `input_ir_act_descriptor_catalog_count`), auto-refreshed every 5s only when socket-connected, with manual refresh.
- Core log polling runs every 3s and pairs `cortex_organ_input` + `cortex_organ_output` into cycle-level cortex cycle cards in Chat view.
- Clicking a cortex cycle card opens a popup that lists per-stage organ activity messages with selectable input/output payload text.
- Chat view keeps a bounded in-memory message ring buffer and incrementally loads older/newer pages on scroll.
- Beluna lifecycle state uses `Hibernate` (instead of `Sleeping`) when Core is unavailable after connection history exists.
- Auth capability descriptors publish explicit payload schemas, including `present_text_message` with a string payload.

### Known Limitations & Mocks

- Organ activity log rendering is polling-based (3s), not filesystem watch-based tail streaming.
- Organ-log pairing relies on `(cycle_id, stage)` FIFO and may skip unmatched events when source files rotate aggressively.
- No in-chat filter/search for cortex cycle cards yet.
- Chat history pagination currently loads from in-memory ring buffer only (no disk-backed history replay).
- Protocol/lifecycle tests should be expanded for metrics and organ-log polling flows.

### Immediate Next Focus

- Add test coverage for metrics polling and organ-log pairing behavior.
- Add in-chat filtering/search for cortex cycle cards and large-payload truncation controls.
- Evaluate filesystem-watch based log streaming to reduce polling latency and repeated tail scans.
