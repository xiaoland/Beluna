# AGENTS.md of Beluna Apple Universal App

Beluna Apple Universal App is the app that bridges human interaction with Beluna Core in Apple ecosystem.

## Tech Stacks

- Language: Swift
- Platform: macOS (for now), iOS, iPadOS
- UI: SwiftUI (use AppKit bridge only when necessary)

## Current Focus

- Harden desktop endpoint UX and observability for Beluna Core integration.

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

> Last Updated At: 2026-02-20T14:46+08:00

### Live Capabilities

- SwiftUI desktop chat endpoint connects to Core via Unix Socket NDJSON (`SpineUnixSocketBodyEndpoint`).
- Connection controls are exposed in `SettingView` (socket path, connect/disconnect, retry).
- Connection intent and socket path are persisted via `UserDefaults`.
- App enforces single-instance runtime lock on macOS.
- Xcode debug sessions default to manual connect.
- Socket path is configured directly from `SettingView` and applied immediately on reconnect.
- Dedicated Observability window browses Core local log files from configurable directory path.
  - Supports manual path apply and macOS folder picker (`NSOpenPanel`).
  - Uses security-scoped bookmark for sandbox-compatible read access.
  - Parses JSON log lines into a timestamp-descending table view.
  - Shows selected row raw line content and keeps tail-read fallback for large files.

### Known Limitations & Mocks

- Observability view is read-only; no search/filter/export yet.
- Log browsing is file snapshot based (refresh/poll), not real-time stream tail.
- Core log format rendering is generic text/JSON, not schema-aware.
- Protocol/lifecycle tests should be expanded for observability flows.

### Immediate Next Focus

- Add test coverage for observability path persistence and file reading behavior.
- Add in-view filtering/search and JSON line helpers for large logs.
- Evaluate non-macOS fallback UX for log directory selection and permissions.
