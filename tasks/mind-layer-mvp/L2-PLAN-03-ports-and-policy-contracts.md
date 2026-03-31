# L2 Plan 03 - Ports And Policy Contracts

- Task Name: `mind-layer-mvp`
- Stage: `L2` / Part 03
- Date: 2026-02-08

## 1) Delegation Port Contract

```rust
pub trait DelegationCoordinatorPort: Send + Sync {
    fn plan(
        &self,
        state: &MindState,
        goal_id: Option<&GoalId>,
    ) -> Result<Vec<ActionIntent>, MindError>;
}
```

Contract requirements:

1. Must be deterministic for same inputs.
2. Must not mutate `MindState` directly.
3. Must not decide preemption/evolution policy.

## 2) Memory Policy Port Contract

```rust
pub enum MemoryDirective {
    Remember {
        key: String,
        summary: String,
    },
    Forget {
        key: String,
        rationale: String,
    },
    KeepTransient,
}

pub trait MemoryPolicyPort: Send + Sync {
    fn decide(
        &self,
        state: &MindState,
        report: &EvaluationReport,
    ) -> Result<MemoryDirective, MindError>;
}
```

Contract requirements:

1. Port can be no-op in MVP via `KeepTransient`.
2. Memory policy concern must stay out of evaluator/resolver logic.
3. Directive output is advisory in MVP (no persistent store required).

## 3) Preemption Policy Contract

```rust
pub struct PreemptionContext<'a> {
    pub state: &'a MindState,
    pub active_goal: Option<&'a GoalRecord>,
    pub incoming_goal: &'a Goal,
    pub safe_point: SafePoint,
}

pub trait PreemptionDecider: Send + Sync {
    fn decide(&self, ctx: PreemptionContext<'_>) -> Result<PreemptionDecision, MindError>;
}
```

Policy constraints:

1. Output disposition must be one of: `Pause`, `Cancel`, `Continue`, `Merge`.
2. `checkpoint_token` is optional and valid only if `preemptable=true`.
3. For `Merge`, decision must include deterministic merge target semantics.

## 4) Safe Point Policy Contract

```rust
pub trait SafePointPolicy: Send + Sync {
    fn inspect(
        &self,
        state: &MindState,
        active_goal_id: Option<&GoalId>,
    ) -> Result<SafePoint, MindError>;
}
```

Policy role:

- central source of preemptability and checkpoint token generation.

## 5) Normative Evaluator Contract

```rust
pub trait NormativeEvaluator: Send + Sync {
    fn evaluate(
        &self,
        state: &MindState,
        command: &MindCommand,
    ) -> Result<EvaluationReport, MindError>;
}
```

Evaluator constraints:

1. Emits criterion-based judgments (alignment/reliability/faithfulness).
2. Must include rationale text for non-`Pass` verdicts.
3. Does not mutate goal lifecycle.

## 6) Conflict Resolver Contract (Scoped Ownership)

```rust
pub trait ConflictResolver: Send + Sync {
    fn resolve(&self, cases: &[ConflictCase]) -> Result<Vec<ConflictResolution>, MindError>;
}
```

Owned conflicts only:

1. helper-output conflicts for same intent,
2. evaluator-verdict conflicts for same criterion window,
3. merge compatibility conflicts.

Non-owned domains:

- goal selection strategy,
- evolution trigger policy,
- transport/runtime errors.

## 7) Evolution Decider Contract

```rust
pub trait EvolutionDecider: Send + Sync {
    fn decide(
        &self,
        state: &MindState,
        evaluation: &EvaluationReport,
    ) -> Result<EvolutionDecision, MindError>;
}
```

Constraints:

1. Proposal-only output.
2. No direct runtime mutation.
3. Rationale + confidence required for `ChangeProposal`.

## 8) Goal Manager Contract

```rust
pub struct GoalManager;

impl GoalManager {
    pub fn register_goal(state: &mut MindState, goal: Goal) -> Result<(), MindError>;
    pub fn activate_goal(state: &mut MindState, goal_id: &GoalId) -> Result<(), MindError>;
    pub fn pause_active_goal(state: &mut MindState, rationale: &str) -> Result<(), MindError>;
    pub fn cancel_goal(state: &mut MindState, goal_id: &GoalId, rationale: &str) -> Result<(), MindError>;
    pub fn merge_goals(
        state: &mut MindState,
        active_goal_id: &GoalId,
        incoming_goal_id: &GoalId,
        merged_goal: Goal,
    ) -> Result<(), MindError>;
    pub fn assert_invariants(state: &MindState) -> Result<(), MindError>;
}
```

## 9) MVP No-op Adapters

```rust
pub struct NoopDelegationCoordinator;
pub struct NoopMemoryPolicy;
```

Behavior:

- delegation returns empty intents,
- memory policy returns `KeepTransient`.

These adapters exist to keep loop wiring explicit while preserving isolation.
