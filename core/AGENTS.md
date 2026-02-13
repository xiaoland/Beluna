# AGENTS.md of Beluna Core

Beluna Core is the runnable runtime and domain agent implementation.

## Tech Stacks

- Language: Rust (2024 edition)

## File Structure (Crucial Only)

```text
.
├── target/
├── Cargo.toml
├── Cargo.lock
├── beluna.schema.json
├── tests/
└── src/
    ├── main.rs
    ├── cli.rs
    ├── core_loop.rs
    ├── body/
    ├── config.rs
    ├── cortex/
    ├── continuity/
    ├── admission/
    ├── ledger/
    ├── spine/
    └── ai_gateway/
```

## Coding Guidelines

- Avoid loose protocol design.
- Use behavior-driven development: User Story -> Acceptance Criteria -> BDT Contract -> Tests -> Implementation.

## Current State

> Last Updated At: 2026-02-13T11:00Z+08:00

### Live Capabilities

- Core runs as a foreground binary: `beluna [--config <path>]`.
- Config defaults to `./beluna.jsonc` and validates against `core/beluna.schema.json`.
- Start the core loop listening on a Unix Socket (NDJSON), exit on SIGTERM/SIGINT.
- Ingest runtime event messages (`sense`, `env_snapshot`, `admission_feedback`, catalog/limits/context updates).
- Run Cortex as an event loop using Continuity ephemeral Sense and Neural Signal queues.
- Batch trigger: sense queue length >= 2 or 1s timeout, max 8 senses per cycle.
- Spine consumes Neural Signals immediately via admission + execution.
- Built-in standard body endpoints (shell/web) run in-process, gated by config and cargo features.
- External body endpoints (for example Apple Universal) register over UnixSocket with `body_endpoint_*` protocol envelopes.
- AI Gateway MVP with deterministic routing, strict normalization, reliability controls, and budget enforcement.

### Known Limitations & Mocks

- WebSocket/HTTP Spine adapters are not implemented in current MVP (UnixSocket adapter only).
- Economic debits from AI Gateway are approximate and currently token-based.
- AI Gateway adapters for cortex extraction/fill rely on model JSON compliance; deterministic clamp remains final authority.

### Immediate Next Focus

- Increase debit fidelity and add persistent ledger storage.
- Continue building macOS Desktop App bridge over Unix Socket.
