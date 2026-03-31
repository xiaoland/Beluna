# Result

## Goal

Align Beluna's documentation system and `AGENTS.md` toward `docs/_svc_v91.md` while preserving the two confirmed Beluna exceptions:

1. `Mode A` keeps a lightweight bypass for brief clarification-only exploration.
2. `Mode C` requires human confirmation only for risky, reference-sensitive, or logic-altering changes.

## Changes Applied

1. Moved the volatile workspace from `docs/task/` to repo-root `tasks/`.
2. Updated path references from `docs/task` to `tasks` across the repository's authoritative and task materials.
3. Updated [`AGENTS.md`](/Users/lanzhijiang/Development/Beluna/AGENTS.md) to adopt a V9.1-style `Mode A / B / C` protocol.
4. Added confirmation gating for:
   - all `Mode B` durable-doc updates,
   - `Mode C` execution only when the change is risky, reference-sensitive, or logic-altering.
5. Updated [`docs/00-meta/index.md`](/Users/lanzhijiang/Development/Beluna/docs/00-meta/index.md), [`docs/00-meta/concepts.md`](/Users/lanzhijiang/Development/Beluna/docs/00-meta/concepts.md), [`docs/10-prd/_drivers/hard-constraints.md`](/Users/lanzhijiang/Development/Beluna/docs/10-prd/_drivers/hard-constraints.md), and [`tasks/README.md`](/Users/lanzhijiang/Development/Beluna/tasks/README.md) to reflect the new separation and workflow.

## Drift Disposition

- `D1 Task workspace location drift`: resolved by migrating to `tasks/`.
- `D2 Task admission and ceremony drift`: intentionally partial drift remains.
  - Beluna now defaults exploratory persisted work to `tasks/`.
  - Beluna still allows brief clarification-only exploration to bypass a task note.
- `D3 Confirmation-gated execution drift`: intentionally partial drift remains.
  - `Mode B` is confirmation-gated.
  - `Mode C` is confirmation-gated only for risky, reference-sensitive, or logic-altering changes.
- `D4 Dynamic mode protocol drift`: resolved by adding a V9.1-style A/B/C execution protocol to `AGENTS.md`.
- `D5 Task cleanup ownership drift`: aligned in lightweight form by asking whether a task note should be archived after completion.

## Verification

- No `docs/task` references remain in the authoritative docs or task workspace.
- Residual `docs/task` references remain only in `scratch/`, which was left untouched because it is non-authoritative.

## Notes

- This is not literal byte-for-byte adoption of `_svc_v91.md`.
- It is a Beluna-local alignment that keeps the framework direction while preserving the two approved exceptions.
