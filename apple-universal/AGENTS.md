# AGENTS.md of Beluna Apple Universal App

Beluna Apple Universal App is the app that bridges human interaction with Beluna Core in Apple ecosystem.

## Tech Stacks

- Language: Swift
- Platform: macOS (for now), iOS, iPadOS
- UI: SwiftUI (use AppKit bridge only when necessary)

## Current Focus

- Build the first desktop UI that communicates with Beluna Core via Unix Socket (NDJSON protocol).

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

> Last Updated At: 2026-02-09T00:00Z+08:00

### Live Capabilities

### Known Limitations & Mocks

### Immediate Next Focus
