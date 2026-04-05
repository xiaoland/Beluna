# Workflow Terms

This file is intentionally small.
It keeps only framework terms that help humans and agents talk about Beluna's documentation system.

Product and domain meaning belongs in [`docs/10-prd/glossary.md`](../10-prd/glossary.md).
Runtime and unit-specific terminology belongs in `20-product-tdd`, `30-unit-tdd`, or local `AGENTS.md` files.

- Typed Input: the front-door classification of work as `Intent`, `Constraint`, `Reality`, or `Artifact`.
- Intent: a request for new product behavior, scope, policy, or strategy.
- Constraint: a technical, dependency, performance, or environment boundary change that does not, by itself, renegotiate product truth.
- Reality: observed behavior that diverges from expectation and requires evidence-first diagnosis.
- Artifact: a bounded intermediate deliverable such as a script, migration helper, report, or one-off analysis output.
- Governing Layer: the authoritative home of the decision being changed.
- Hard Unit: a unit whose local complexity deserves durable design memory in `30-unit-tdd`.
- Local AGENTS: closest-to-target tactical guardrails and recurrence tripwires for a code area.
- Task Packet: a bounded task note in `tasks/` used to absorb volatility without polluting durable docs.
- MVT Anchors: `Objective & Hypothesis`, `Guardrails Touched`, and `Verification`.
- Explore: mode for mapping unknowns, alternatives, and assumptions.
- Solidify: mode for turning findings into explicit claims, contracts, or decisions.
- Execute: mode for implementing a clear, verified change.
- Diagnose: mode for evidence-first investigation of a reality mismatch.
- Promotion: moving stable truth from a task or discussion into its durable owner.
- Demotion: simplifying or deleting durable text that no longer earns its keep.
