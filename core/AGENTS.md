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
    ├── config.rs
    ├── afferent_pathway.rs
    ├── types.rs                 # Sense, Act, Neural-Signal Descriptor and other shared types
    ├── stem.rs
    ├── cortex/                  # The highness, cognition engine of Beluna
    ├── continuity/
    ├── ledger/
    ├── spine/
    ├── body/
    ├── observaiblity/
    └── ai_gateway/
```

## Coding Guidelines

- Prohibit implicit fallbacks and inline default assignments used to mask missing states.
- Use `concat!` to split long strings across multiple lines for better readability without introducing unwanted whitespace or runtime overhead.

## Current State

> Last Updated At: 2026-02-21T16:40+08:00

### Live Capabilities

- Core runs as a foreground binary: `beluna [--config <path>]`.
- Config defaults to `./beluna.jsonc` and validates against `core/beluna.schema.json`.
- Runtime uses one bounded Rust MPSC sense queue with native sender backpressure.
- `main` boots continuity/spine/cortex, starts the Stem loop, and listens for SIGTERM/SIGINT.
- Shutdown closes ingress gate first, then blocks until `hibernate` sense is enqueued.
- Runtime logging is `tracing`-only: JSON file logs with rotation/retention and optional stderr warn/error mirroring via `logging.*` config.
- Stem is tick-driven (`loop.tick_interval_ms`, default 10s) with missed tick skip behavior.
- Stem can invoke Cortex with empty domain senses on timer ticks.
- Stem waits for incoming sense before next Active tick only when Cortex output declares `wait_for_sense=true` (derived from Primary `<is-wait-for-sense>` tag).
- Stem composes physical+cognition state, invokes pure Cortex boundary, persists returned cognition state, then dispatches acts serially through Continuity -> Spine.
- Control senses:
  - `hibernate` breaks loop immediately.
  - `new_neural_signal_descriptors` / `drop_neural_signal_descriptors` mutate capability state before same-cycle Cortex call.
- Stem exposes a built-in act descriptor `core.control/sleep` with payload `{seconds}` for timed sleep mode.
- Continuity persists cognition state (`goal-tree` + `l1-memory`) to JSON and enforces deterministic guardrails.
- Continuity and Spine both hold afferent-pathway sender handles.
- Spine `on_act` emits dispatch-failure senses to afferent pathway on reject/error.
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
