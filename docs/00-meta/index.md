# Beluna Documentation System

Beluna documentation is organized by stability and decision scope.

## Authoritative Layers

1. `docs/00-meta/`: terminology, documentation rules, and governance.
2. `docs/10-prd/`: pressure-driven product intent (`_drivers -> behavior -> domain-structure`).
3. `docs/20-product-tdd/`: system-level technical realization.
4. `docs/30-unit-tdd/`: unit-level technical realization.
5. `docs/40-deployment/`: deployment and runtime operational truth.

## Non-Authoritative Area

- `docs/task/` is procedural workspace for tasks and plans.
- Task files are not source of truth.
- Stable outcomes discovered in tasks must be promoted into authoritative layers above.

## How To Read

1. Read [`read-order.md`](./read-order.md) for the default cross-layer loading order.
2. Read [`concepts.md`](./concepts.md) for ontology and terminology.
3. Read [`intake-protocol.md`](./intake-protocol.md) before planning or implementation.
4. Read [`promotion-rules.md`](./promotion-rules.md) before promoting transient outcomes.
5. Read [`doc-system.md`](./doc-system.md) for document families and stability placement.
6. Read PRD in order: `_drivers` first, `behavior` second, `domain-structure` third.
7. Read `40-deployment` for runtime constraints.
8. Read relevant `docs/task/<task>/` files for transient context only.
9. Use [`legacy-triage.md`](./legacy-triage.md) only for migration history and archival mapping.
