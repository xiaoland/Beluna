# tasks Workspace

`tasks/` is the repo-root volatile workspace for plans, investigations, and result notes.
It is intentionally separated from the authoritative layers under `docs/`.

> **Non-authoritative.** Nothing here governs Beluna's behavior.
> Promote stable outcomes into the proper durable layer before relying on them.

## When To Use It

Use a task folder by default for high-volatility exploration when the work needs persisted context, comparison space, or temporary coordination.
Large ambiguous work should usually open a task note.
Short clarification-only exploration may bypass a task note.
Straightforward low-risk execution work does not need task docs.

## Minimal Workflow

1. Capture enough context to keep the work coherent.
2. Record only the notes you actually need.
3. If exploration hardens into durable product or technical truth, stop and get human confirmation before updating authoritative docs.
4. If execution becomes risky, reference-sensitive, or logic-altering, stop and get human confirmation before coding.
5. Verify the change.
6. Promote stable truths into `10/20/30/40` or code/tests when appropriate.

Existing `L0/L1/L2/L3` task packs are historical deep-work conventions, not the default workflow.

## Promotion Test

Promote a task finding only when it is:

- stable
- reusable beyond the current task
- costly or risky to rediscover
- not better enforced mechanically

## Suggested Shape

- `PLAN.md` for working notes when needed
- `RESULT.md` for closure when needed
- extra files only when the task is genuinely large

Start from [`_template.md`](./_template.md) if you want a lightweight note.

## Durable Destinations

- Product what/why: `docs/10-prd`
- Cross-unit technical truth: `docs/20-product-tdd`
- Hard-unit local design memory: `docs/30-unit-tdd`
- Runtime and operational truth: `docs/40-deployment`
- Mechanically enforced rules: code/tests/CI

Delete or ignore leftover task detail once it stops being useful.
When a task note directly drove the finished change, ask whether it should be archived.
