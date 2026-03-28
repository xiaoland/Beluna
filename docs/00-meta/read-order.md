# Read Order

This file provides an advisory loading strategy for humans and agents to reduce drift and overreach.

Use the smallest relevant slice; read locally first, then apply global kernel governance.

## Typical Path (Advisory)

1. Nearest relevant `AGENTS.md` (root first, then local refinements).
2. Relevant layer `index.md` (`10-prd`, `20-product-tdd`, `30-unit-tdd`, or `40-deployment`).
3. Relevant layer documents.
4. `docs/00-meta/concepts.md`, `docs/00-meta/intake-protocol.md`, `docs/00-meta/promotion-rules.md`, and `docs/00-meta/doc-system.md` when terminology/governance/promotion questions are in scope.
5. Relevant `docs/task/<task>/*` (transient context only).
6. Code and tests.

## Notes

- There is no universal mandatory read ritual; choose what is needed for the task.
- For PRD reads, keep internal order: `_drivers` -> `behavior` -> `glossary` -> `domain-structure`.
- If layers conflict, outer layer wins: PRD -> Product TDD -> Unit TDD -> deployment/unit operations -> code details.
- `docs/task` never overrides authoritative layers.
- `docs/00-meta/legacy-triage.md` is archival and should not be used as governing truth.
