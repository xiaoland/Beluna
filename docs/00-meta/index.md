# Meta Index

`docs/00-meta` is Beluna's small cross-layer coordination surface.
It is Beluna-local, not a mandatory baseline for every repository.

For most implementation work, do not start here.
Start from the nearest `AGENTS.md`, then the authoritative layer that owns the decision.

## What `00-meta` Owns

- Cross-layer operating terms and the decision-network workflow.
- Minimal read guidance for ambiguous or cross-layer work.
- Intake rules for non-trivial perturbations.
- Promotion and demotion rules for durable docs.
- Boundaries of the documentation system itself.

## What `00-meta` Does Not Own

- Product/domain semantics or glossary truth: `docs/10-prd`.
- Cross-unit technical realization: `docs/20-product-tdd`.
- Unit-local contracts: `docs/30-unit-tdd`.
- Runtime operations: `docs/40-deployment`.
- Task reasoning and execution notes: `docs/task`.

## Open `00-meta` Only When Needed

Use this folder when one or more apply:

1. The request is cross-layer or ambiguous.
2. You are changing documentation policy, ownership, or promotion rules.
3. The task needs explicit perturbation classification or task-packet discipline.
4. You are resolving drift between layers or removing stale durable docs.

## Files

- [`concepts.md`](./concepts.md): cross-layer terms and the decision-network model.
- [`read-order.md`](./read-order.md): minimal loading strategy.
- [`intake-protocol.md`](./intake-protocol.md): classify and contain non-trivial work.
- [`promotion-rules.md`](./promotion-rules.md): promote, demote, and place durable truth.
- [`doc-system.md`](./doc-system.md): ownership boundary of `00-meta` versus `10/20/30/40`.
- [`legacy-triage.md`](./legacy-triage.md): archival migration context only.
