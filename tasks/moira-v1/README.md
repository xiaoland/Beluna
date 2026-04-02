# Moira V1 Task Buffer

This packet is the non-authoritative workspace for decomposing the Moira upgrade before code and durable docs move in lockstep.

## Working Frame

- Product shape: `Moira` is a binary desktop application; `Loom` is its human-facing UI control plane.
- Internal roles:
  - `Clotho`: Core artifact download, hash verification, version isolation, config preparation.
  - `Lachesis`: OTLP logs ingest, local storage, query, and visualization.
  - `Atropos`: Core wake, stop, and force-kill control.
- Current working assumptions:
  - macOS-first.
  - new `/moira` unit instead of rewriting `/monitor`.
  - Tauri + Rust + Vue/TypeScript.
  - logs-first observability.
  - DuckDB as the embedded log store.
  - GitHub prereleases as the primary Core artifact source.
  - JSONC-only config editing.
  - Moira manages Beluna Core only in v1, not all endpoint apps.
  - quitting Moira also stops the supervised Core.
  - force-kill requires a second confirmation step.
  - local development can point Clotho at a Core source folder and compile before launch.
  - product-facing terminology should converge from `cycle_id` to `tick`.
  - goal-forest comparison is derived by comparing two ticks, not by persisting a precomputed diff record.

## Packet Map

- `L0.md`: intake packet, governing anchors, guardrails, and task outcome stance.
- `L1.md`: scope split, workstream map, and sequencing strategy.
- `L2.md`: architecture seams, candidate module boundaries, and event-model expectations.
- `L3.md`: execution slices designed for incremental landing.
- `STATUS.md`: current implementation snapshot and session handoff anchor.
- `BACKEND-REFACTOR-TARGET.md`: cleanup-stage backend landing target for the Moira Unit TDD backend module split.
- `FRONTEND-REFACTOR-TARGET.md`: cleanup-stage frontend landing target for the Loom layer split.
- `CLEANUP-LANDING-PLAN.md`: concrete cleanup slices, authoritative TDD touchpoints, and local AGENTS guidance.
- `LACHESIS.md`: collection model and visualization primitives for observability-first design.
- `OPEN-QUESTIONS.md`: unresolved decisions and deferred negotiation points.

## Workflow Note

- This packet follows the Beluna task workflow: `L0` context, `L1` high-level strategy, `L2` low-level design, `L3` implementation plan.
- The numbered stages in `L3.md` and `LACHESIS.md` are implementation stages of this single task.
- They are not separate product-release phases or roadmap buckets.

## Use Rule

- Anything in this folder is provisional until implementation and promotion happen together.
- If a conclusion becomes stable and recurring, promote it into `docs/10-prd`, `docs/20-product-tdd`, `docs/30-unit-tdd`, or `docs/40-deployment`.
- If implementation pressure pushes toward a choice that harms readability or maintainability, stop and negotiate before encoding it as a task assumption.
