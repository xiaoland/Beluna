# L1 Plan 08 - Docs and Contracts Refresh
- Task: `cortex-loop-architecture`
- Micro-task: `08-docs-contracts-refresh`
- Stage: `L1`
- Date: `2026-03-01`
- Status: `DRAFT_FOR_APPROVAL`

## High-Level Strategy
1. Refresh authoritative docs only after code contracts are stabilized by micro-tasks `01..07`.
2. Update docs from contracts first, then module/topology narratives, then AGENTS and final result file.
3. Treat task docs as procedural notes, not architecture truth.

## Architectural Design
1. Contract updates first:
- sense schema and rendering contract
- runtime ownership/invocation invariants
- act tooling and bounded waiting semantics
- reset transaction semantics.
2. Module docs second:
- stem, cortex, continuity, spine, body, ai-gateway references impacted by flow changes.
3. Operational docs third:
- AGENTS updates for invariants/non-goals
- `RESULT.md` with boundary deltas and rationale.

## Key Technical Decisions
1. One canonical deterministic metadata rendering block is authored once and referenced from other docs.
2. Examples must include text payload, `weight`, and optional `act_instance_id`.
3. Doc drift sweep uses targeted search for stale terms (`metadata` field, `wait_for_sense: bool`, `expand-sense-raw`, direct Stem->Cortex invocation).

## Dependency Requirements
1. Micro-task `02` must be complete before finalizing sense contracts.
2. Micro-tasks `04`, `05`, `06`, `07` must be complete before topology diagrams are finalized.
3. Final docs update is the completion gate for this task and feeds `RESULT.md`.

## L1 Exit Criteria
1. Impacted doc inventory and update ordering are explicit.
2. Contract-level wording for new behavior is deterministic and testable.
3. Ready to draft L2 doc-level change maps.
