# Unit TDD Index

## Role in the system

`30-unit-tdd` defines local technical realization per unit under Product TDD constraints.

It is the authoritative layer for unit-local responsibilities, interfaces, data/state assumptions, operations, and verification contracts.

## What this layer owns

1. Unit-local responsibility and non-responsibility boundaries.
2. Unit-local interface contracts and dependency assumptions.
3. Unit-local data/state ownership and invariants.
4. Unit-local operational rules.
5. Unit-local verification contracts, guardrails, and evidence homes.

## What must NOT appear here

1. Product drivers, claims, or other PRD truth (belongs to `10-prd`).
2. Cross-unit decomposition policy (belongs to `20-product-tdd/unit-boundary-rules.md`).
3. Cross-unit authority ownership changes (belongs to `20-product-tdd/system-state-and-authority.md`).
4. Cross-unit contract redefinition (belongs to `20-product-tdd/cross-unit-contracts.md`).
5. Deployment runbook truth (belongs to `40-deployment`).

## How to read this layer

1. Read this index first to understand Unit TDD scope and escalation rules.
2. Read the unit `README.md`.
3. Read `design.md` and `interfaces.md`.
4. Read `data-and-state.md` and `operations.md`.
5. Read `verification.md` before making or reviewing changes.

## How this layer connects to adjacent layers

1. Inherits system constraints and boundaries from `20-product-tdd`.
2. Supplies implementation-facing unit contracts used by code/tests.
3. Defers runtime operational procedures to `40-deployment`.
4. Consumes and supports claim realization defined in `20-product-tdd/claim-realization-matrix.md`.

## Common local mistakes

1. Redefining cross-unit boundaries inside a unit doc.
2. Mixing unit-local assumptions with product-level claims.
3. Leaving unit data/state ownership implicit.
4. Leaving verification expectations implicit.
5. Treating task notes as unit contract truth.

## Unit TDD Index Contract

Unit docs are allowed to decide:

1. local implementation structure inside an existing unit boundary.
2. local interfaces and operations consistent with upstream contracts.
3. local data/state assumptions and verification rules.

Unit docs must escalate upward when a change affects:

1. unit decomposition policy or split/merge decisions.
2. cross-unit contracts or compatibility expectations.
3. system authority ownership.
4. unit-to-container mapping.

Required per-unit structure:

- `README.md`
- `design.md`
- `interfaces.md`
- `data-and-state.md`
- `operations.md`
- `verification.md`

Anti-overreach interpretation rules (agents/reviewers):

1. If a statement changes behavior across multiple units, treat it as Product TDD scope first.
2. If a statement changes cross-unit contract shape or authority ownership, do not approve it as unit-local truth.
3. If a statement governs runtime rollout/recovery procedure, place it in `40-deployment` rather than Unit TDD.

## Unit Catalog

- [Core Unit](./core/README.md)
- [CLI Unit](./cli/README.md)
- [Apple Universal Unit](./apple-universal/README.md)
