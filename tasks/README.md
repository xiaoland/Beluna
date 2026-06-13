# tasks Workspace

`tasks/` is the repo-root agent-owned, task-local volatile workspace for plans, investigations, diagnostics, evidence, collaboration state, and tactical artifacts.
It is intentionally separated from the authoritative layers under `docs/`.

> **Non-authoritative.** Nothing here governs Beluna's behavior.
> Promote stable outcomes into the proper durable layer before relying on them.

## When To Use It

Use a task packet by default for non-trivial work that needs persisted context, comparison space, diagnostics, evidence, or temporary coordination.
Large ambiguous work should usually open a task workspace.
Short clarification-only exploration may bypass a task packet.
Straightforward low-risk localized execution work does not need task docs.

## Task Packet Invariants

A task packet is a bounded workspace, not just a note file.

- Agent-owned: the agent may create, update, split, and reorganize packet files inside the task boundary without separate approval.
- Task-local: temporary reasoning, scratch artifacts, exploration notes, and verification material stay inside the bounded workspace.
- Human-agent-collaboration-oriented: the packet remains readable, inspectable, and steerable by the human.
- Recoverable: a resumed agent can restore current state from a compact control surface.
- Bounded: the packet serves one task and does not become a permanent knowledge base.
- Non-durable: packet contents are not source truth until they pass the promotion test and move to the correct owner.
- Search-isolated: volatile task material is excluded from ordinary source and durable-doc search by default.

Agent-owned does not change the ownership rules for code, durable docs, public configuration, or generated release artifacts.

## MVT Core

Every non-trivial task packet should include these three anchors:

- `Objective & Hypothesis`: the goal and the expected effect of the work
- `Guardrails Touched`: the 1-2 existing rules or boundaries that must not be violated
- `Verification`: objective proof that the work is done correctly

These are guardrails, not bureaucracy.

## Current State

Keep the compact control surface current enough that another agent or the human can quickly answer:

- what the agent currently believes is true
- what constraints or decisions the human has confirmed
- what mode or transition explains the current posture
- what the next concrete action is

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
- `Supporting Files`

Existing `L0/L1/L2/L3` task packs are historical deep-work conventions, not the default workflow.

## Minimal Workflow

1. Capture the MVT anchors.
2. Keep current understanding, confirmed constraints, and next step current when losing that state would increase task risk.
3. Record only the notes you actually need.
4. Split supporting files by collaboration pressure, not ceremony.
5. If exploration hardens into durable product or technical truth, switch to `Solidify` and get human confirmation before updating authoritative docs.
6. If execution becomes risky, reference-sensitive, logic-altering, or non-obviously local, switch to `Execute` with the Impact Handshake before mutation.
7. Verify the change.
8. Promote only stable truths into `10/20/30/40`, local `AGENTS.md`, or code/tests when appropriate.

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

## Search Isolation

Exclude `tasks/` from ordinary source and durable-doc search by default.
Search it only when the active question targets task state, when recovering work, or when reviewing evidence deliberately stored here.

## Suggested Shape

Start single-file when the packet is compact.

Upgrade to directory mode when current state, historical reasoning, evidence, decisions, temporary work, or verification begin to interfere with each other.

Recommended directory-mode starting point:

```text
tasks/<task-id>/
|-- packet.md
|-- notes.md
`-- work/
```

Split `notes.md` by pressure, not ceremony. Common split directions include current versus history, state versus evidence, decision versus exploration, control surface versus work, and summary versus raw output.

Existing `PLAN.md` / `RESULT.md` files are still valid historical packets. New work should prefer the compact `packet.md` control surface when directory mode is needed.

Start from [`_template.md`](./_template.md) if you want a lightweight task packet.

## Durable Destinations

- Product what or why: `docs/10-prd`
- Cross-unit technical truth: `docs/20-product-tdd`
- Hard-unit local design memory: `docs/30-unit-tdd`
- Runtime and operational truth: `docs/40-deployment`
- Mechanically enforced rules: code, tests, type systems, CI
