# AGENTS.md of Beluna Core

Beluna Core is the runtime and domain agent implementation.

## Tech Stacks

- Language: Rust (2024 edition)

## File Structure (Crucial Only)

```text
.
├── target/
├── Cargo.toml
├── Cargo.lock
├── beluna.jsonc
├── beluna.schema.json
├── tests/
└── src/
    ├── main.rs
    ├── cli.rs
    ├── config.rs
    ├── mind/
    ├── protocol.rs
    ├── server.rs
    └── ai_gateway/
```

## Coding Guidelines

- Avoid Loose protocol design.
- Uses Behavior-Driven Development: User Story -> Acceptance Criteria -> BDT Contract -> Tests -> Implementation.

## Current State

> Last Updated At: 2026-02-08T14:10Z+08:00

### Live Capabilities

- Load config (jsonc, with JSONSchema support)
- Start the core loop listening on an Unix Socket (NDJSON), exit on SIGTERM or exit message.
- AI Gateway MVP, a thin boundary that standardizes how the runtime calls external AI backends and receives results.
- Mind layer MVP, with deterministic goal management, preemption, evaluation, conflict handling, and proposal-only evolution decisions.

### Known Limitations & Mocks

- Current design is likely lacking of interactivity, lost Human-in-loop.
- Gateway is implemented, not yet integrated into the system.
- Mind is internal-only in MVP and does not interact with Unix socket protocol/runtime directly.

### Immediate Next Focus

- A simple macOS Desktop App that bridges human and Beluna (use the Unix Socket), the very first UI of Beluna.
