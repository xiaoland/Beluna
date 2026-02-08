# AGENTS.md of Beluna

Beluna is an agent.

## Tech Stacks

- Language: Rust (2024 edition)

## File Structure (Crucial Only)

```text
.
├── target/
├── docs/                # Product, BDT Contracts, Modules, ADR and Tasks
├── tests/
└── src/
    ├── main.rs
    ├── cli.rs
    ├── config.rs
    ├── protocol.rs
    ├── server.rs
    └── ai_gateway/
```

## Documents

Read following documents if needed, and keep them current:

- [Overview](./docs/overview.md): Product overview.
- [Glossary](./docs/glossary.md): Product top-level glossary.
- [Feature Document](./docs/features/README.md): Feature document, answers "What to do", each feature has its PRD, HLD and LLD.
- [Modules](./docs/modules/README.md)
- [BDT Contract](./docs/contracts/README.md)
- [ADR](./docs/descisions/README.md)

> Note that documents are for communication only, code are the single source of truth.
> You are encouraged to add an AGENTS.md file under modules with significant complexity when needed.

## Coding Guidelines

- Avoid Loose protocol design.
- Uses Behavior-Driven Development: User Story -> Acceptance Criteria -> BDT Contract -> Tests -> Implementation.

## Current State

> Last Updated At: 2026-02-08T14:10Z+08:00

### Live Capabilities

- Load config (jsonc, with JSONSchema support)
- Start the core loop listening on an Unix Socket (NDJSON), exit on SIGTERM or exit message.
- AI Gateway MVP, a thin boundary that standardizes how the runtime calls external AI backends and receives results.

### Known Limitations & Mocks

- Gateway is implemented, not yet integrated into the system.

### Immediate Next Focus

- A simple MacOS Desktop App that bridges human and Beluna (use the Unix Socket), the very first UI of Beluna.
