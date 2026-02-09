# Mind HLD

## High-Level Architecture

Mind is implemented as an internal control kernel with strict dependency inversion.

```text
Runtime Integrator (future)
   -> MindFacade
      -> MindState
      -> GoalManager
      -> SafePointPolicy + PreemptionDecider
      -> DelegationCoordinatorPort
      -> NormativeEvaluator
      -> ConflictResolver
      -> MemoryPolicyPort
      -> EvolutionDecider
      -> MindEvent / MindDecision output
```

## Boundary Principles

1. Mind does not depend on Unix socket runtime or protocol modules.
2. External execution concerns enter only through ports.
3. Mind owns authoritative goal and decision state.

## Component Model

- `MindFacade`: deterministic cycle orchestration.
- `MindState`: continuity state container.
- `GoalManager`: lifecycle + invariants.
- `Preemption`: policy-constrained disposition decisions.
- `Evaluator`: normative judgments.
- `ConflictResolver`: scoped deterministic conflict handling.
- `MemoryPolicyPort`: remember/forget policy boundary.
- `EvolutionDecider`: proposal-only evolution decisioning.

## Runtime Guarantees

- deterministic processing for identical input/state,
- single active goal invariant,
- closed preemption disposition set,
- no direct body mutation in MVP.

## Involved Surfaces

- `src/mind/mod.rs`
- `src/mind/error.rs`
- `src/mind/types.rs`
- `src/mind/state.rs`
- `src/mind/goal_manager.rs`
- `src/mind/preemption.rs`
- `src/mind/evaluator.rs`
- `src/mind/conflict.rs`
- `src/mind/evolution.rs`
- `src/mind/ports.rs`
- `src/mind/noop.rs`
- `src/mind/facade.rs`
