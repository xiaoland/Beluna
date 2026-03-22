# Beluna Documentation System

Beluna documentation is organized by stability and decision scope.

## Authoritative Layers

1. `docs/00-meta/`: terminology, documentation rules, and governance.
2. `docs/10-prd/`: product intent and product-level invariants.
3. `docs/20-product-tdd/`: system-level technical realization.
4. `docs/30-unit-tdd/`: unit-level technical realization.
5. `docs/40-deployment/`: deployment and runtime operational truth.
6. `docs/90-decisions/`: ADR history.

## Non-Authoritative Area

- `docs/task/` is procedural workspace for tasks and plans.
- Task files are not source of truth.
- Stable outcomes discovered in tasks must be promoted into authoritative layers above.

## How To Read

1. Start with [`concepts.md`](./concepts.md).
2. Read [`doc-system.md`](./doc-system.md) to understand update rules.
3. Review migration outcomes in [`legacy-triage.md`](./legacy-triage.md).
4. Read `10-prd` before `20-product-tdd` and `30-unit-tdd`.
5. Read `40-deployment` for runtime constraints.
6. Read ADRs only for decision history and rationale; operative conclusions should already be reflected in TDD layers.
