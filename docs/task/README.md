# docs/task Workspace

`docs/task` is an active procedural workspace for plans, execution notes, and result logs.

## Authority Rule (Soft Quarantine)

1. `docs/task` is **non-authoritative**.
2. Task files may reference authoritative docs, but authoritative docs must not depend on task files for core definitions.
3. Stable outcomes discovered in tasks must be promoted to one of:
- `docs/10-prd`
- `docs/20-product-tdd`
- `docs/30-unit-tdd`
- `docs/40-deployment`

## Complex Task Workflow

1. Analyze request and context (`L0`).
2. Draft high-level strategy (`L1`).
3. Draft low-level design (`L2`).
4. Draft implementation plan (`L3`).
5. Execute implementation against approved plan.
6. Record task result.

## File Convention

- Use `docs/task/<task-name>/`.
- Keep planning stages as `L0/L1/L2/L3` files when needed.
- Keep final outcome as `RESULT.md` when needed.

## Quality Notes

- Prefer clear reasoning over procedural verbosity.
- Do not treat temporary trade-offs as durable truth.
- If a repeated rule appears across tasks, promote it into authoritative docs.
- Historical task files may reference removed legacy docs; treat those references as archival context only.
