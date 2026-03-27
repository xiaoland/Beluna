# Read Order

This file defines the default loading order for humans and agents to reduce drift and overreach.

Use the smallest relevant slice; read locally first, then apply global kernel governance.

## Default Order

1. Nearest relevant `AGENTS.md` (root first, then local refinements).
2. Relevant layer `index.md` (`10-prd`, `20-product-tdd`, `30-unit-tdd`, or `40-deployment`).
3. `docs/00-meta/concepts.md`.
4. `docs/00-meta/intake-protocol.md`.
5. `docs/00-meta/promotion-rules.md`.
6. `docs/00-meta/doc-system.md`.
7. Relevant layer documents.
8. Relevant `docs/task/<task>/*` (transient context only).
9. Code and tests.

## Notes

- For PRD reads, keep internal order: `_drivers` -> `behavior` -> `domain-structure`.
- If layers conflict, outer layer wins: PRD -> Product TDD -> Unit TDD -> deployment/unit operations -> code details.
- `docs/task` never overrides authoritative layers.
- `docs/00-meta/legacy-triage.md` is archival and should not be used as governing truth.
