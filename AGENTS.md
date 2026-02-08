# AGENTS.md of Beluna

Beluna is an agent.

## Tech Stacks

- Language: Rust (2024 edition)

## File Structure (Crucial Only)

```text
.
├── target/
├── docs/
├── tests/
└── src/
    ├── main.rs
    ├── cli.rs
    ├── config.rs
    ├── protocol.rs
    ├── server.rs
    └── ai_gateway/
        └── AGENTS.md
```

## Documentations

Read following documents if needed, and keep them current:

- [Product Design](./docs/product/README.md): Product overview, glossary, feature documents involves user story, acceptance criteria.
- [BDT Contract](./docs/contract/README.md)
- [AI Gateway](./src/ai_gateway/AGENTS.md)

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

- Gateway is implemented is not yet integrated into the system.

### Immediate Next Focus

- An interactive shell that bridges human and Beluna (use the Unix Socket), the very first UI of Beluna.
