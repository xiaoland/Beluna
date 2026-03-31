# L0 Plan 06 - State Ownership and Continuity Refactor
- Task: `cortex-loop-architecture`
- Micro-task: `06-state-ownership-and-continuity-refactor`
- Stage: `L0`
- Date: `2026-03-01`
- Status: `DRAFT_FOR_APPROVAL`

## Objective
Reassign physical/cognition state responsibilities according to new runtime model.

## Scope
1. Physical state stays Stem-owned.
2. Expose physical state as `Arc<RwLock<PhysicalState>>`.
3. Ensure `<proprioception>` section is refreshed before dispatching new senses to Primary.
4. Cortex calls Continuity directly for cognition persistence.
5. Remove cognition snapshot/persist responsibilities from Stem.
6. Remove L1 memory from cognition model and helper/prompt/IR paths.
7. Keep goal-forest helper on a normal DI lifecycle (non-singleton).
8. Make goal-forest `to_input_ir` deterministic without LLM transformation.

## Current State
1. Stem builds physical state by value each cycle.
2. Stem snapshots/persists cognition via Continuity.
3. L1 memory is active in cognition state, prompts, IR, helpers.
4. Goal-forest helper still uses LLM transform for patch conversion path.

## Target State
1. Shared physical state lock is authoritative.
2. Cognition persistence flow is Cortex -> Continuity.
3. L1 memory is fully removed.
4. Goal-forest helper is deterministic utility managed by DI without singleton coupling.

## Key Gaps
1. Shared-state synchronization model is missing.
2. Continuity API surface for Cortex-owned persistence must be redesigned.
3. Large cleanup required across cognition/prompt/IR/helper files for L1 removal.

## Risks
1. Concurrency bugs around `Arc<RwLock<PhysicalState>>`.
2. Partial L1 removal may leave inconsistent IR/prompt paths.

## L0 Exit Criteria
1. Final ownership map for physical/cognition state is explicit.
2. L1 removal impact map is complete.
3. Deterministic non-singleton goal-forest helper contract is defined.
