# Beluna Documentation System

Beluna documentation is organized by stability and decision scope.
This is a Beluna-local operating model, not a universal requirement for every repository.

## Beluna Authoritative Layers

1. `docs/00-meta/`: terminology, documentation rules, and governance.
2. `docs/10-prd/`: pressure-driven product intent (`_drivers -> behavior -> glossary -> domain-structure`).
3. `docs/20-product-tdd/`: system-level technical realization.
4. `docs/30-unit-tdd/`: unit-level technical realization.
5. `docs/40-deployment/`: deployment and runtime operational truth.

## Non-Authoritative Area

- `docs/task/` is procedural workspace for tasks and plans.
- Task files are not source of truth.
- Stable outcomes discovered in tasks must be promoted into authoritative layers above.

## How To Read (Contextual)

Use the smallest relevant slice for the change you are making.

Typical path:

1. Start from nearest relevant `AGENTS.md`.
2. Read the relevant layer index (`10/20/30/40`).
3. Read [`read-order.md`](./read-order.md) if scope is cross-layer or ambiguous.
4. Read [`concepts.md`](./concepts.md), [`intake-protocol.md`](./intake-protocol.md), [`promotion-rules.md`](./promotion-rules.md), and [`doc-system.md`](./doc-system.md) when terminology/governance/promotion decisions are involved.
5. Read PRD in order: `_drivers` first, `behavior` second, `glossary` third, `domain-structure` fourth.
6. Read relevant `docs/task/<task>/` files for transient context only.
7. Use [`legacy-triage.md`](./legacy-triage.md) only for migration history and archival mapping.
