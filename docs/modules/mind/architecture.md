# Mind Architecture

## Structure

- `MindFacade`: deterministic cycle orchestrator.
- `MindState`: in-process continuity state.
- `GoalManager`: invariant enforcement for goal lifecycle.
- `PreemptionDecider` + `SafePointPolicy`: goal-switch policy boundary.
- `NormativeEvaluator`: criterion-based judgments.
- `ConflictResolver`: scoped conflict handling.
- `DelegationCoordinatorPort`: helper delegation planning port.
- `MemoryPolicyPort`: remember/forget policy port.
- `EvolutionDecider`: proposal-only evolution decisioning.

## Dependency Direction

- `src/mind/*` depends only on local mind modules and shared primitives.
- `src/mind/*` must not depend on `src/spine/adapters/unix_socket_runtime.rs`, `src/spine/adapters/wire.rs`, or `src/ai_gateway/*`.

## Determinism Rules

- stable map/set ordering,
- no random/wall-clock control logic,
- deterministic tie-breaks for conflicts and merges.
