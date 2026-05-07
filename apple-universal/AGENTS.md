# AGENTS.md of Beluna Apple Universal

Beluna Apple Universal is the Apple ecosystem Beluna Human Interface.

## Tech Stacks

- Language: Swift
- Platform: macOS (for now), iOS, iPadOS
- UI: SwiftUI (use AppKit bridge only when necessary)

## Boundary and Quality Rules

- Own Apple-native endpoint UX and the first minimum native Moira Loom.
- Core retains runtime behavior, endpoint protocol authority, and observability emission semantics.
- Moira retains local preparation, supervision, observability ingestion/storage/query/projection, and future platform adapter semantics.
- Treat socket protocol compatibility as a contract; prefer explicit typed request/response mapping.
- Keep UI responsive: socket I/O, Moira runtime calls, and parsing stay off the main thread.
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
- Settings-integrated Moira Loom ownership boundaries.
- Resource conflict presentation for embedded process-local Moira runtime.

## Issue 30 Direction

1. The first Moira Loom surface lands as a Settings-integrated operations panel.
2. Apple Universal first uses process-local embedded Moira runtime.
3. Body endpoint socket discovery remains a first-class path for Core processes started elsewhere.
4. Source cleanup should establish dedicated Settings sections and smaller state owners before adding Moira UI.
