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
    ├── cortex/
    ├── non_cortex/
    ├── spine/
    ├── protocol.rs
    ├── server.rs
    └── ai_gateway/
```

## Coding Guidelines

- Avoid loose protocol design.
- Use behavior-driven development: User Story -> Acceptance Criteria -> BDT Contract -> Tests -> Implementation.

## Current State

> Last Updated At: 2026-02-11T14:30Z+08:00

### Live Capabilities

- Load config (jsonc, with JSONSchema support).
- Start the core loop listening on a Unix Socket (NDJSON), exit on SIGTERM or exit message.
- AI Gateway MVP with deterministic routing, strict normalization, reliability controls, and budget enforcement.
- Cortex + Non-cortex + Spine contracts:
  - Cortex owns goals/commitments and emits deterministic, non-binding `IntentAttempt[]`.
  - Non-cortex admits/denies mechanically, maintains survival ledger, and reconciles settlements.
  - Spine executes admitted actions only and returns ordered, replayable execution events.

### Known Limitations & Mocks

- Gateway is implemented, not yet wired into the runtime socket loop.
- Spine is contract-level MVP (execution adapter is deterministic noop in tests/default path).
- Economic debits from AI Gateway are approximate and currently token-based.

### Immediate Next Focus

- Wire Cortex/Non-cortex/Spine loop into runtime entrypoints.
- Increase debit fidelity and add persistent ledger storage.
- Continue building macOS Desktop App bridge over Unix Socket.
