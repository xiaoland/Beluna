# AGENTS.md of Beluna Core

Beluna Core is the runnable runtime and domain agent implementation.

## Tech Stack

- Language: Rust (2024 edition)

## Local Boundaries

- Core is the runnable authority owner; body endpoints and monitor-side tools must not re-implement its domain decisions.
- Cross-unit contracts, authority boundaries, and deployment-shaping constraints are owned by `docs/20-product-tdd/*`.
- Core-local implementation constraints and verification expectations are owned by `docs/30-unit-tdd/core/*`.

## Coding

- Prohibit implicit fallbacks and inline default assignments used to mask missing states.
- Use `concat!` to split long strings across multiple lines for better readability without introducing unwanted whitespace or runtime overhead.

## High-Risk Areas

- Runtime authority boundaries between `cortex`, `stem`, `continuity`, and `spine`.
- Afferent/efferent ordering, backpressure, and shutdown semantics.
- Wire contract and compatibility between Core and body endpoints.
- Observability/export policy consistency with deployment docs.
