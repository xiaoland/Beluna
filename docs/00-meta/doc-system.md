# Documentation System Rules

Beluna uses a layered documentation system to preserve alignment without excessive maintenance cost.

## Stability Placement

| Stability | Layers | Examples |
|---|---|---|
| **Slow** | `00-meta`, `10-prd`, `20-product-tdd`, major `30-unit-tdd` boundaries | terminology, product invariants, unit topology |
| **Medium** | unit interfaces, operations, `40-deployment` runbooks | interface contracts, deployment steps |
| **Fast** | `docs/task` plans and in-progress notes | task L0–L3 files, scratch analysis |
| **Executable** | codebase | tests, schemas, runtime validation |

Slow-layer statements must survive multiple task cycles without meaningful change.
Medium-layer statements should remain stable within a release cycle.
Fast-layer content is ephemeral and expires with the task that produced it.
Executable truth must always match authoritative layer contracts; divergence is a defect.

## Update Workflow

1. Classify the change by scope: terminology, product intent, cross-unit design, unit-local design, deployment/runtime, or task-local.
2. Update the nearest stable anchor first.
3. Implement the change in code/config.
4. Verify behavior and operational impact.
5. Promote only stable learning back into authoritative docs.

## Promotion Gate (Required)

A statement can be promoted from `docs/task` or from code into an authoritative layer only if all checks pass:

1. It matches current implementation/runtime behavior.
2. It is stable truth, not temporary execution detail.
3. It belongs to exactly one target layer.
4. It improves clarity and lowers ambiguity.

### Promotion Targets

| Discovery | Promote To |
|---|---|
| New canonical term or concept | `00-meta/concepts.md` |
| Product intent or invariant | `10-prd` |
| Cross-unit design decision | `20-product-tdd` |
| Unit-local design decision | `30-unit-tdd/<unit>` |
| Runtime or operational constraint | `40-deployment` |
| Significant design decision with rationale | `90-decisions` (ADR) |

### QA Cross-Cutting Guidance

When validating a change that spans multiple layers:

1. Confirm the outer-layer statement (PRD invariant or product-tdd contract) is satisfied.
2. Confirm unit-level contracts (interfaces, operations) remain consistent.
3. Confirm executable truth (tests, schema) enforces the contract.
4. If any layer is inconsistent, fix the authoritative layer first, then propagate downward.

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
