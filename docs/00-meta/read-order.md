# Read Order

This file provides a minimal loading strategy for humans and agents.
Use the smallest relevant slice and avoid turning documentation into a mandatory ritual.

## Default Path

1. Read the nearest relevant `AGENTS.md`.
2. Read the authoritative layer `index.md` that owns the decision:
   - `10-prd` for product truth
   - `20-product-tdd` for cross-unit design
   - `30-unit-tdd` for unit-local contracts
   - `40-deployment` for runtime operations
3. Read the exact authoritative files you need.
4. Read code/tests for executable truth.
5. Read `docs/task/<task>/` only for transient context.

## Open `00-meta` Only on These Triggers

Read `docs/00-meta` when one or more apply:

- The request is cross-layer or reference-sensitive.
- Ownership between layers is unclear.
- You need the intake protocol or task-packet structure.
- You need promotion/demotion rules.
- You are changing the doc system itself.

## Notes

- There is no universal mandatory read ritual.
- For PRD reads, keep internal order: `_drivers` -> `behavior` -> `glossary` -> `domain-structure`.
- If layers conflict, outer layer wins: PRD -> Product TDD -> Unit TDD -> deployment/unit operations -> code details.
- `docs/task` never overrides authoritative layers.
- `docs/00-meta/legacy-triage.md` is archival and should not be used as governing truth.
