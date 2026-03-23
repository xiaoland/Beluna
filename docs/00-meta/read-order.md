# Read Order

This file defines the default loading order for humans and agents to reduce drift and overreach.

Use the smallest relevant slice; do not read unrelated layers by default.

## Default Order

1. Nearest relevant `AGENTS.md` (root, then local refinements).
2. `docs/00-meta/concepts.md`.
3. `docs/00-meta/doc-system.md`.
4. `docs/00-meta/intake-protocol.md`.
5. Relevant `docs/10-prd/_drivers/*`.
6. Relevant `docs/10-prd/behavior/*`.
7. Relevant `docs/10-prd/domain-structure/*` (derived structure, not upstream truth).
8. Relevant `docs/20-product-tdd/*`.
9. Relevant `docs/30-unit-tdd/<unit>/*`.
10. Relevant `docs/40-deployment/*`.
11. Relevant `docs/task/<task>/*` (transient context only).
12. Code and tests.

## Notes

- If layers conflict, outer layer wins: PRD -> Product TDD -> Unit TDD -> deployment/unit operations -> code details.
- `docs/task` never overrides authoritative layers.
- `docs/00-meta/legacy-triage.md` is archival and should not be used as governing truth.
