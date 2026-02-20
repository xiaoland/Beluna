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
    ├── body/
    ├── config.rs
    ├── cortex/
    ├── continuity/
    ├── ingress.rs
    ├── ledger/
    ├── runtime_types.rs
    ├── spine/
    ├── stem.rs
    └── ai_gateway/
```

## Coding Guidelines

- Avoid loose protocol design.
- Use behavior-driven development: User Story -> Acceptance Criteria -> BDT Contract -> Tests -> Implementation.

## Current State

> Last Updated At: 2026-02-20T11:30+08:00

### Live Capabilities

- Core runs as a foreground binary: `beluna [--config <path>]`.
- Config defaults to `./beluna.jsonc` and validates against `core/beluna.schema.json`.
- Runtime uses one bounded Rust MPSC sense queue with native sender backpressure.
- `main` boots continuity/ledger/spine/cortex, starts the Stem loop, and listens for SIGTERM/SIGINT.
- Shutdown closes ingress gate first, then blocks until `sleep` sense is enqueued.
- Runtime logging is `tracing`-only: JSON file logs with rotation/retention and optional stderr warn/error mirroring via `logging.*` config.
- Stem consumes one sense at a time, composes physical+cognition state, invokes pure cortex boundary, then dispatches acts serially through Ledger -> Continuity -> Spine.
- Control senses:
  - `sleep` breaks loop without calling Cortex.
  - `new_capabilities` / `drop_capabilities` mutate capability state before same-cycle Cortex call.
- Built-in inline body endpoints (shell/web under `core/src/body`) are started by `main` after Spine boot, each on a dedicated thread, and attach through Spine Inline Adapter configured in `spine.adapters`.
- External body endpoints register over UnixSocket and publish senses/capability patches.
- AI Gateway MVP provides deterministic routing, strict normalization, reliability controls, budget enforcement, and tracing-structured telemetry events.

### Known Limitations & Mocks

- WebSocket/HTTP spine adapters are not implemented in current MVP (UnixSocket adapter only).
- Economic debits from AI Gateway are approximate and currently token-based.
- AI Gateway adapters for cortex extraction/fill rely on model JSON compliance; deterministic clamp remains final authority.

### Immediate Next Focus

- Increase debit fidelity and add persistent ledger storage.
- Continue building macOS Desktop App bridge over UnixSocket.
