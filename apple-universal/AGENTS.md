# AGENTS.md of Beluna Apple Universal App

Beluna Apple Universal App is the app that bridges human interaction with Beluna Core in Apple ecosystem.

## Local Use

- Follow [`../AGENTS.md`](../AGENTS.md) for the default workflow and layer routing.
- Use this file only for Apple Universal local boundaries, risks, and coding constraints.

## Tech Stacks

- Language: Swift
- Platform: macOS (for now), iOS, iPadOS
- UI: SwiftUI (use AppKit bridge only when necessary)

## Boundary and Quality Rules

- Keep desktop logic as a Body Endpoint of Core; do not re-implement Core domain logic in desktop.
- Treat socket protocol compatibility as a contract; prefer explicit typed request/response mapping.
- Keep UI responsive: socket I/O and parsing must not block the main thread.
- Add tests for protocol decoding/encoding and connection lifecycle behavior where practical.

## Design Sources

- Product drivers and behavior claims: `../docs/10-prd/index.md`
- System and unit boundaries: `../docs/20-product-tdd/index.md`, `../docs/30-unit-tdd/index.md`
- Deployment and operations constraints: `../docs/40-deployment/index.md`

## Stability Boundary

- Keep durable boundary and quality constraints in this file; avoid volatile runtime capability snapshots.
- Current behavior/status/focus should live in task notes or release notes.
- Treat `docs/30-unit-tdd/apple-universal/*`, `docs/20-product-tdd/*`, and `docs/40-deployment/*` as authoritative for evolving contracts.

## High-Risk Areas

- Socket protocol compatibility with Core-side contracts.
- Connection lifecycle and reconnection behavior under failure.
- Main-thread responsiveness under socket I/O and parsing load.
- Local persistence and history pagination behavior for large chat state.
