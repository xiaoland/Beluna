# Documentation System Rules

Beluna uses a layered documentation system to preserve alignment without excessive maintenance cost.

## Stability Placement

- Slow-changing truths: `00-meta`, `10-prd`, `20-product-tdd`, major `30-unit-tdd` boundaries.
- Medium-changing truths: unit interfaces, operations, and deployment runbooks.
- Fast-changing truths: task plans and in-progress notes in `docs/task`.
- Executable truths: code, tests, schemas, and validation rules in the codebase.

## Update Workflow

1. Classify the change by scope: terminology, product intent, cross-unit design, unit-local design, deployment/runtime, or task-local.
2. Update the nearest stable anchor first.
3. Implement the change in code/config.
4. Verify behavior and operational impact.
5. Promote only stable learning back into authoritative docs.

## Promotion Gate (Required)

A legacy statement can be promoted only if all checks pass:

1. It matches current implementation/runtime behavior.
2. It is stable truth, not temporary execution detail.
3. It belongs to exactly one target layer.
4. It improves clarity and lowers ambiguity.

## Task Quarantine (`docs/task`)

- `docs/task` is allowed and active, but non-authoritative.
- Durable docs may reference a task as evidence, not as governing truth.
- If a task conclusion becomes repeatedly relevant, promote it to `10/20/30/40` layers.

## ADR Usage

- ADRs preserve decision memory and rationale.
- Current operative conclusions must be reflected in `20-product-tdd` or `30-unit-tdd`.
- Reviewers should understand current system state without reading full ADR history.

## Removed Legacy Families

The following legacy families were removed from the authoritative map:

- `docs/features`
- `docs/modules`
- `docs/contracts`
- `docs/overview.md`
- `docs/glossary.md`
- `docs/descisions` (replaced by `docs/90-decisions`)

Do not reintroduce these as parallel authoritative systems.
