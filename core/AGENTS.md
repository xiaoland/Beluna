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
    ├── continuity/
    ├── admission/
    ├── ledger/
    ├── spine/
    ├── protocol.rs
    ├── server.rs
    └── ai_gateway/
```

## Coding Guidelines

- Avoid loose protocol design.
- Use behavior-driven development: User Story -> Acceptance Criteria -> BDT Contract -> Tests -> Implementation.

## Current State

> Last Updated At: 2026-02-11T23:40Z+08:00

### Live Capabilities

- Load config (jsonc, with JSONSchema support).
- Start the core loop listening on a Unix Socket (NDJSON), exit on SIGTERM or exit message.
- Ingest runtime event messages (`sense`, `env_snapshot`, `admission_feedback`, catalog/limits/context updates).
- Run Cortex as an always-on reactor task with bounded inbox/outbox channels.
- AI Gateway MVP with deterministic routing, strict normalization, reliability controls, and budget enforcement.
- Cortex + Continuity + Admission + Ledger + Spine contracts:
  - Cortex consumes `ReactionInput` and emits deterministic, non-binding `IntentAttempt[]`.
  - Cortex requires `based_on` grounding and deterministic `attempt_id` derivation.
  - Continuity ingests feedback and builds non-semantic `SituationView`.
  - Admission performs deterministic effectuation gating.
  - Ledger enforces survival resource accounting and settlement terminality.
  - Spine executes admitted actions only and returns ordered, replayable execution events.

### Known Limitations & Mocks

- Spine is contract-level MVP (execution adapter is deterministic noop in tests/default path).
- Economic debits from AI Gateway are approximate and currently token-based.
- AI Gateway adapters for cortex extraction/fill rely on model JSON compliance; deterministic clamp remains final authority.

### Immediate Next Focus

- Increase debit fidelity and add persistent ledger storage.
- Continue building macOS Desktop App bridge over Unix Socket.
