# L2 Plan 02 - Domain Model And Invariants

- Task Name: `mind-layer-mvp`
- Stage: `L2` / Part 02
- Date: 2026-02-08

## 1) Core Identifiers

```rust
pub type GoalId = String;
pub type IntentId = String;
pub type CycleId = u64;
pub type CheckpointToken = String;
```

## 2) Goal Model

```rust
pub enum GoalLevel {
    High,
    Mid,
    Low,
}

pub enum GoalStatus {
    Proposed,
    Active,
    Paused,
    Cancelled,
    Completed,
    Merged,
}

pub struct Goal {
    pub id: GoalId,
    pub title: String,
    pub level: GoalLevel,
    pub parent_goal_id: Option<GoalId>,
    pub priority: u8,
    pub created_cycle: CycleId,
    pub metadata: std::collections::BTreeMap<String, String>,
}

pub struct GoalRecord {
    pub goal: Goal,
    pub status: GoalStatus,
    pub merged_into: Option<GoalId>,
}
```

## 3) Safe Point Model (Minimal)

```rust
pub struct SafePoint {
    pub preemptable: bool,
    pub checkpoint_token: Option<CheckpointToken>,
    pub rationale: String,
}
```

Safe point appears in:

- `PreemptionDecision` output,
- optional `ActionIntent` metadata for downstream consumers.

## 4) Preemption Decision Types

```rust
pub enum PreemptionDisposition {
    Pause,
    Cancel,
    Continue,
    Merge,
}

pub struct PreemptionDecision {
    pub disposition: PreemptionDisposition,
    pub rationale: String,
    pub safe_point: SafePoint,
    pub merge_goal_id: Option<GoalId>,
}
```

## 5) Evaluation Model (Normative)

```rust
pub enum EvaluationCriterion {
    GoalAlignment,
    SubsystemReliability,
    SignalFaithfulness,
}

pub enum EvaluationVerdict {
    Pass,
    Borderline,
    Fail,
    Unknown,
}

pub struct Judgment {
    pub criterion: EvaluationCriterion,
    pub verdict: EvaluationVerdict,
    pub confidence: f32,
    pub rationale: String,
    pub evidence_refs: Vec<String>,
}

pub struct EvaluationReport {
    pub goal_id: Option<GoalId>,
    pub judgments: Vec<Judgment>,
}
```

## 6) Delegation and Intent Model

```rust
pub enum IntentKind {
    Delegate,
    Evaluate,
    Observe,
    ProposeEvolution,
}

pub struct ActionIntent {
    pub intent_id: IntentId,
    pub goal_id: Option<GoalId>,
    pub kind: IntentKind,
    pub description: String,
    pub checkpoint_token: Option<CheckpointToken>,
    pub metadata: std::collections::BTreeMap<String, String>,
}

pub struct DelegationResult {
    pub intent_id: IntentId,
    pub helper_id: String,
    pub payload: serde_json::Value,
    pub confidence: f32,
}
```

## 7) Conflict Model (Scoped)

```rust
pub enum ConflictCase {
    HelperOutputSameIntent {
        intent_id: IntentId,
        candidates: Vec<DelegationResult>,
    },
    EvaluatorVerdictSameCriterion {
        criterion: EvaluationCriterion,
        candidates: Vec<Judgment>,
    },
    MergeCompatibility {
        active_goal_id: GoalId,
        incoming_goal_id: GoalId,
    },
}

pub enum ConflictResolution {
    SelectedHelperResult { intent_id: IntentId, helper_id: String },
    SelectedJudgment { criterion: EvaluationCriterion, verdict: EvaluationVerdict },
    MergeAllowed { merged_goal_id: GoalId },
    MergeRejected,
    NoConflict,
}
```

## 8) Evolution Model (Proposal-only)

```rust
pub enum EvolutionTarget {
    Model { id: String },
    MemoryStructure { id: String },
    PerceptionPipeline { id: String },
}

pub enum EvolutionAction {
    Replace,
    Retrain,
    Reconfigure,
}

pub struct ChangeProposal {
    pub target: EvolutionTarget,
    pub action: EvolutionAction,
    pub rationale: String,
    pub evidence_refs: Vec<String>,
    pub confidence: f32,
}

pub enum EvolutionDecision {
    NoChange { rationale: String },
    ChangeProposal(ChangeProposal),
}
```

## 9) Mind I/O Model

```rust
pub enum MindCommand {
    ProposeGoal(Goal),
    ObserveSignal {
        signal_id: String,
        fidelity_hint: Option<f32>,
        payload: serde_json::Value,
    },
    SubmitDelegationResult(DelegationResult),
    EvaluateNow,
}

pub enum MindEvent {
    GoalActivated { goal_id: GoalId },
    GoalPaused { goal_id: GoalId },
    GoalCancelled { goal_id: GoalId },
    GoalMerged { from_goal_id: GoalId, into_goal_id: GoalId },
    PreemptionDecided { disposition: PreemptionDisposition },
    EvaluationCompleted,
    ConflictResolved,
    EvolutionDecided,
}

pub enum MindDecision {
    Preemption(PreemptionDecision),
    DelegationPlan(Vec<ActionIntent>),
    Evaluation(EvaluationReport),
    Conflict(ConflictResolution),
    Evolution(EvolutionDecision),
}

pub struct MindCycleOutput {
    pub cycle_id: CycleId,
    pub events: Vec<MindEvent>,
    pub decisions: Vec<MindDecision>,
}
```

## 10) MindState Definition

```rust
pub struct MindState {
    pub cycle_id: CycleId,
    pub goals: std::collections::BTreeMap<GoalId, GoalRecord>,
    pub active_goal_id: Option<GoalId>,
    pub pending_intents: std::collections::VecDeque<ActionIntent>,
    pub recent_evaluations: std::collections::VecDeque<EvaluationReport>,
    pub recent_delegation_results: std::collections::VecDeque<DelegationResult>,
    pub last_preemption: Option<PreemptionDecision>,
}
```

## 11) Invariants

1. Single active goal invariant
- At most one goal has `GoalStatus::Active`.
- If `active_goal_id = Some(id)`, that goal record exists and is `Active`.

2. Parent relation invariant
- Mid/low goals with `parent_goal_id` must reference an existing goal.

3. Merge invariant
- `Merged` goals must set `merged_into = Some(target_id)`.
- merged goal cannot remain active.

4. Checkpoint invariant
- `checkpoint_token` may appear only when `safe_point.preemptable == true`.

5. Confidence invariant
- all confidence values must be clamped to `[0.0, 1.0]`.

6. Determinism invariant
- same `(MindState, MindCommand, deterministic ports/policies)` must produce identical `MindCycleOutput`.

## 12) State Transition Summary

```text
Proposed -> Active -> Paused -> Active -> Completed
Proposed -> Active -> Cancelled
Proposed -> Active -> Merged (points to merged goal)
```

No direct transition from `Completed/Cancelled/Merged` back to `Active`.
