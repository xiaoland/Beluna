# AGENTS.md of Beluna Core

Beluna Core is the runnable runtime and domain agent implementation.

## Tech Stacks

- Language: Rust (2024 edition)

## Design Sources

- Product intent/invariants: `../docs/10-prd/index.md`
- System-level design: `../docs/20-product-tdd/index.md`
- Core unit design/interfaces/operations: `../docs/30-unit-tdd/core/README.md`
- Deployment constraints: `../docs/40-deployment/index.md`
- Decision history: `../docs/90-decisions/README.md`

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
    ├── types.rs                 # Sense, Act, Neural-Signal Descriptor and other shared types
    ├── stem.rs
    ├── stem/
    │   ├── runtime.rs
    │   ├── afferent_pathway.rs
    │   └── efferent_pathway.rs
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

> Last Updated At: 2026-03-22T16:45+08:00

### Live Capabilities

- Core runs as a foreground binary: `beluna [--config <path>]`.
- Config defaults to `./beluna.jsonc` and validates against `core/beluna.schema.json`.
- Runtime uses one bounded Rust MPSC sense queue with native sender backpressure.
- `main` boots continuity/spine/cortex, starts Stem tick runtime and Cortex runtime on separate tasks, and listens for SIGTERM/SIGINT.
- Shutdown closes ingress gate and cancels runtime tasks (no `hibernate` control sense).
- Runtime logging is `tracing`-only with dual sinks: JSON file logs named `core.log.<YYYY-MM-DD>.<awake_sequence>` (retention cleanup + optional stderr warn/error via `logging.*`) and OTLP log export via `observability.otlp.signals.logs.*`.
- Runtime metrics use OpenTelemetry OTLP export via `observability.otlp.signals.metrics.*`; Prometheus pull endpoint is removed.
- Runtime traces use OpenTelemetry OTLP export via `observability.otlp.signals.traces.*` with configurable parent-based ratio sampling.
- Stem is tick-driven (`loop.tick_interval_ms`, default 10s) and only emits tick grants.
- Cortex owns admitted cognition cycle execution (tick-gated with buffered senses) and consumes the afferent receive handle in its own runtime task.
- Cortex Primary persists cognition state through Continuity directly from cognition tools.
- Cortex Primary act tools dispatch directly into the efferent FIFO and receive `ActDispatchResult` (bounded wait; timeout -> `lost`).
- Cortex Primary act tools emit per-act `wait_for_sense` integer seconds (`0` means no wait) and use unified `expand-senses`.
- Stem owns physical state mutation for `ns_descriptor`/proprioception through `StemControlPort` + shared store.
- Control senses were removed from afferent flow; descriptor/proprioception updates are direct runtime control calls.
- Continuity persists cognition state (`goal-forest`) to JSON and enforces deterministic guardrails.
- Afferent queue carries domain senses only and runs a deferral scheduler before Cortex consumption.
- Deferral rules are managed by pathway-owned control API:
  - `add_rule` inserts one rule by `rule_id` (duplicate id is rejected).
  - `remove_rule` removes one rule by `rule_id`.
  - `min_weight` selector defers when `sense.weight < min_weight`.
  - `fq-sense-id` selector defers when regex matches `endpoint_id/neural_signal_descriptor_id`.
- Deferred senses are buffered FIFO with `loop.max_deferring_nums` cap; overflow evicts oldest deferred entries with warnings.
- Afferent sidecar is observe-only and emits rule/defer/release/eviction events.
- Efferent shutdown drains with bounded timeout (`loop.efferent_shutdown_drain_timeout_ms`) before forced drop.
- Spine `on_act_final` emits dispatch-failure senses to afferent pathway on reject/lost.
- Built-in inline body endpoints (shell/web under `core/src/body`) are started by `main` after Spine boot, each on a dedicated thread, and attach through Spine Inline Adapter configured in `spine.adapters`.
- External body endpoints register over UnixSocket and publish text-payload senses (`payload`, `weight`, optional `act_instance_id`).
- Adapter-initiated descriptor/proprioception updates flow `adapter -> Spine runtime -> StemControlPort` (adapters do not call Stem directly).
- AI Gateway MVP provides deterministic routing, strict normalization, reliability controls, budget enforcement, and tracing-structured telemetry events.

### Known Limitations & Mocks

- WebSocket/HTTP spine adapters are not implemented in current MVP (UnixSocket adapter only).
- Economic debits from AI Gateway are approximate and currently token-based.
- AI Gateway adapters for cortex extraction/fill rely on model JSON compliance; deterministic clamp remains final authority.

### Immediate Next Focus

- Increase debit fidelity and add persistent ledger storage.
- Continue building macOS Desktop App bridge over UnixSocket.
