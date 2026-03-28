# L3 PLAN - Execution Sequence

## Sequence
1. Create `DECISIONS.md` and lock disposition for each drift item.
2. Land Package A (governance/read-path) in one PR.
3. Land Package B (semantics ownership) in one PR.
4. Land Package C (Unit TDD admission) in one PR.
5. Land Package D (AGENTS scope control) in one PR.
6. Land Package E (demotion lifecycle) in one PR if not already included.
7. Run cross-layer consistency review and record `RESULT.md`.

## Rollback Guidance
- If a package creates cross-layer contradiction, revert that package only.
- Do not partially revert individual statements that create orphan policy fragments.

## Exit Criteria
1. Drift register item statuses are explicit and justified.
2. No contradictory ownership language across `00/10/20/30/40`.
3. AGENTS docs are practical and bounded.
4. Demotion rule exists and is referenced by maintainers.
