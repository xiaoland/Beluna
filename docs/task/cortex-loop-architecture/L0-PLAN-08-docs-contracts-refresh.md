# L0 Plan 08 - Docs and Contracts Refresh
- Task: `cortex-loop-architecture`
- Micro-task: `08-docs-contracts-refresh`
- Stage: `L0`
- Date: `2026-03-01`
- Status: `DRAFT_FOR_APPROVAL`

## Objective
Synchronize module/contract/agent documentation with the refactored topology.

## Scope
1. Update module docs for Stem/Cortex/Spine/Continuity/Body and topography diagrams.
2. Update contracts for:
- loop authority and invocation invariants
- pathway ownership and lifecycle
- act tool-calling and bounded wait semantics
- sense text payload + weight + optional act_instance_id contract.
3. Update AGENTS files with new invariants and non-goals.
4. Produce final `docs/task/cortex-loop-architecture/RESULT.md`.

## Current State
1. Docs still describe Stem direct invocation and old act/sense models in multiple places.
2. Contracts do not yet encode reset-thread surgery and new tooling model.

## Target State
1. Docs and code boundaries are aligned.
2. Contracts are testable and unambiguous for future tasks.

## Key Gaps
1. Broad drift across modules/contracts/agents.
2. Cross-file terminology inconsistencies.

## Risks
1. Outdated docs can reintroduce invalid assumptions in later refactors.

## L0 Exit Criteria
1. All impacted docs are listed with update intent.
2. Contract deltas are explicitly mapped to runtime behavior changes.
