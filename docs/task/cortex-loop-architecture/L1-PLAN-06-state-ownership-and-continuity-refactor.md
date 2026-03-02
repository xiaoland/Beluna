# L1 Plan 06 - State Ownership and Continuity Refactor
- Task: `cortex-loop-architecture`
- Micro-task: `06-state-ownership-and-continuity-refactor`
- Stage: `L1`
- Date: `2026-03-01`
- Status: `DRAFT_FOR_APPROVAL`

## High-Level Strategy
1. Keep physical state ownership in Stem but expose shared read access through `Arc<RwLock<PhysicalState>>`.
2. Move cognition persistence trigger from Stem to Cortex through a Continuity persistence port.
3. Remove `l1_memory` from cognition model and all runtime/prompt/helper dependencies.
4. Keep goal-forest helper as a deterministic DI-managed utility without singleton/shared-runtime semantics.

## Architectural Design
1. Physical state model:
- Stem mutates canonical state
- Cortex reads snapshot before each Primary dispatch
- `<proprioception>` section is refreshed from latest shared state before sending senses.
2. Cognition persistence model:
- Cortex commits new cognition state via continuity port after each turn
- Stem no longer snapshots or persists cognition.
3. Cognition shape simplification:
- keep goal forest
- remove focal awareness (`l1_memory`) paths from IR, prompts, helpers, and limits.
4. Goal-forest helper model:
- regular Cortex dependency injected by DI (non-singleton)
- deterministic `to_input_ir` and deterministic patch application semantics.

## Key Technical Decisions
1. Shared lock policy is explicit: short-lived read locks in Cortex, write locks only in Stem mutation paths.
2. Continuity integration is port-based to avoid direct concrete dependency from Cortex to `ContinuityEngine` internals.
3. No LLM-based transformation remains in goal-forest conversion path.
4. Removing `l1_memory` is a full cut, not soft deprecation.

## Dependency Requirements
1. Micro-task `01` DI boundary must exist first.
2. Micro-task `05` reset flow should consume deterministic goal-forest renderer from this task.
3. Continuity storage and validation must accept new cognition shape without `l1_memory`.

## L1 Exit Criteria
1. Physical and cognition ownership boundaries are explicit.
2. `l1_memory` removal scope is fully enumerated.
3. Deterministic goal-forest helper strategy is locked.
