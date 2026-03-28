# AGENTS.md of Beluna Core

Beluna Core is the runnable runtime and domain agent implementation.

## Tech Stacks

- Language: Rust (2024 edition)

## Design Sources

- Product drivers/behavior claims: `../docs/10-prd/index.md`
- System-level design: `../docs/20-product-tdd/index.md`
- Core unit design/interfaces/operations: `../docs/30-unit-tdd/core/README.md`
- Deployment constraints: `../docs/40-deployment/index.md`

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

## Stability Boundary

- Keep durable constraints in this file; avoid storing volatile runtime capability snapshots here.
- Current runtime behavior, limitations, and near-term focus should live in task notes or release notes.
- Treat `docs/30-unit-tdd/core/*`, `docs/20-product-tdd/*`, and `docs/40-deployment/*` as authoritative for evolving behavior contracts.

## High-Risk Areas

- Runtime authority boundaries between `cortex`, `stem`, `continuity`, and `spine`.
- Afferent/efferent ordering, backpressure, and shutdown semantics.
- Wire contract and compatibility between Core and body endpoints.
- Observability/export policy consistency with deployment docs.
