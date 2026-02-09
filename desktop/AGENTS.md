# AGENTS.md of Beluna Desktop

Beluna Desktop is the macOS app that bridges human interaction with Beluna Core.

## Tech Stacks

- Language: Swift
- Platform: macOS
- UI: SwiftUI (use AppKit bridge only when necessary)

## Current Focus

- Build the first desktop UI that communicates with Beluna Core via Unix Socket (NDJSON protocol).

## Boundary and Quality Rules

- Keep desktop logic as a client of Core; do not re-implement Core domain logic in desktop.
- Treat socket protocol compatibility as a contract; prefer explicit typed request/response mapping.
- Keep UI responsive: socket I/O and parsing must not block the main thread.
- Add tests for protocol decoding/encoding and connection lifecycle behavior where practical.

## Design Sources

- Product overview: `../docs/overview.md`
- Shared glossary: `../docs/glossary.md`
- Contracts and protocol context: `../docs/contracts/README.md`

## Last Updated

> 2026-02-09
