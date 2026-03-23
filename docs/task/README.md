# docs/task Workspace

`docs/task` is an active procedural workspace for plans, execution notes, and result logs.

> **Non-authoritative.** Nothing in `docs/task` governs Beluna's design or behavior.
> Stable outcomes discovered here must be promoted into the authoritative layers before they can be relied upon.

## Authority Rule (Soft Quarantine)

1. `docs/task` is **non-authoritative**.
2. Task files may reference authoritative docs, but authoritative docs must not depend on task files for core definitions.
3. Stable outcomes discovered in tasks must be promoted to one of:
- `docs/10-prd`
- `docs/20-product-tdd`
- `docs/30-unit-tdd`
- `docs/40-deployment`

## When to Promote

Promote a task outcome when it meets **all** of the following:

- The conclusion is likely to recur or apply beyond this task.
- It matches current implementation/runtime behavior (or will once the task lands).
- It belongs clearly to one authoritative layer (see [Promotion Targets](../00-meta/doc-system.md#promotion-targets)).
- It improves clarity without duplicating existing statements.

If in doubt, record the outcome in `RESULT.md` first and promote in a follow-up PR.

## Complex Task Workflow

1. Analyze request and context (`L0`).
2. Draft high-level strategy (`L1`).
3. Draft low-level design (`L2`).
4. Draft implementation plan (`L3`).
5. Execute implementation against approved plan.
6. Record task result.
7. **Promote** stable findings to authoritative docs.

## Task Verification Packet

For non-trivial tasks, keep a lightweight packet with:

- **Governing Anchors**: stable docs the task depends on.
- **Intended Change**: concise statement of scope.
- **Acceptance Criteria**: what must be true to call the task done.
- **Guardrails Touched**: tests, schemas, CI checks, rollout checks involved.
- **Evidence Expected**: proof needed before closure.
- **Promotion Candidates**: recurring truths to push back into `10/20/30/40`.

## File Convention

- Use `docs/task/<task-name>/`.
- Keep planning stages as `L0/L1/L2/L3` files when needed.
- Keep final outcome as `RESULT.md` when needed.

## Quality Notes

- Prefer clear reasoning over procedural verbosity.
- Do not treat temporary trade-offs as durable truth.
- If a repeated rule appears across tasks, promote it into authoritative docs.
- Historical task files may reference removed legacy docs; treat those references as archival context only.

## QA Cross-Cutting Guidance

Before closing a task, verify across layers:

1. Any new product invariant is reflected in `10-prd`.
2. Any cross-unit design change is reflected in `20-product-tdd`.
3. Any unit contract change is reflected in `30-unit-tdd`.
4. Executable truth (tests, schemas) enforces the updated contracts.

If a layer is inconsistent, fix it before marking the task complete.
