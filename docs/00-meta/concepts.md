# Workflow Terms

This file is intentionally small.
It keeps only repo-level workflow terms that help humans and agents talk about Beluna's documentation system.

Product and domain meaning belongs in [`docs/10-prd/glossary.md`](../10-prd/glossary.md).
Runtime and unit-specific terminology belongs in `20-product-tdd` or the relevant `30-unit-tdd` unit.

- Governing Layer: the authoritative home of the decision being changed.
- Hard Unit: a unit whose local complexity deserves durable design memory in `30-unit-tdd`.
- Task Note: volatile planning, investigation, or result context kept under `tasks/`.
- Mode A / Exploration: high-volatility work buffered in `tasks/` when it needs persisted context; brief clarification-only exploration may stay ephemeral.
- Mode B / Solidification: clear but unrecorded durable truth that requires human confirmation before durable doc updates.
- Mode C / Execution: clear localized implementation work; human confirmation is required only for risky, reference-sensitive, or logic-altering changes.
- Promotion: moving stable truth from a task or discussion into its durable owner.
- Demotion: simplifying or deleting durable text that no longer earns its keep.
