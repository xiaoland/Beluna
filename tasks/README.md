# tasks Workspace

`tasks/` is the repo-root volatile workspace for plans, investigations, diagnostics, and tactical artifacts.
It is intentionally separated from the authoritative layers under `docs/`.

> **Non-authoritative.** Nothing here governs Beluna's behavior.
> Promote stable outcomes into the proper durable layer before relying on them.

## When To Use It

Use a task packet by default for non-trivial work that needs persisted context, comparison space, diagnostics, or temporary coordination.
Large ambiguous work should usually open a task note.
Short clarification-only exploration may bypass a task note.
Straightforward low-risk localized execution work does not need task docs.

## MVT Core

Every non-trivial task packet should include these three anchors:

- `Objective & Hypothesis`: the goal and the expected effect of the work
- `Guardrails Touched`: the 1-2 existing rules or boundaries that must not be violated
- `Verification`: objective proof that the work is done correctly

These are guardrails, not bureaucracy.

## Optional Exploration Scaffold

Use these only when they help reduce ambiguity:

- `Perturbation`
- `Input Type`
- `Active Mode or Transition Note`
- `Governing Anchors`
- `Impact Hypothesis`
- `Temporary Assumptions`
- `Negotiation Triggers`
- `Promotion Candidates`

Existing `L0/L1/L2/L3` task packs are historical deep-work conventions, not the default workflow.

## Minimal Workflow

1. Capture the MVT anchors.
2. Record only the notes you actually need.
3. If exploration hardens into durable product or technical truth, switch to `Solidify` and get human confirmation before updating authoritative docs.
4. If execution becomes risky, reference-sensitive, or logic-altering, switch to `Execute` with the restatement gate before coding.
5. Verify the change.
6. Promote only stable truths into `10/20/30/40` or code/tests when appropriate.

## Promotion Test

Promote a task finding only when it is:

- stable
- reusable beyond the current task
- costly or risky to rediscover
- not better enforced mechanically

## Exit Rule

Do not let task notes become shadow architecture.
Delete, archive, or ignore leftover task detail once it stops being useful.
When a task note directly drove the finished change, ask whether it should be archived.

## Suggested Shape

- `PLAN.md` for working notes when needed
- `RESULT.md` for closure when needed
- extra files only when the task is genuinely large

Start from [`_template.md`](./_template.md) if you want a lightweight task packet.

## Durable Destinations

- Product what or why: `docs/10-prd`
- Cross-unit technical truth: `docs/20-product-tdd`
- Hard-unit local design memory: `docs/30-unit-tdd`
- Runtime and operational truth: `docs/40-deployment`
- Mechanically enforced rules: code, tests, type systems, CI
