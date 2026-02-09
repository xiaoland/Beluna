# L2 Plan 04 - Deterministic Loop And Algorithms

- Task Name: `mind-layer-mvp`
- Stage: `L2` / Part 04
- Date: 2026-02-08

## 1) MindFacade Loop

```rust
impl MindFacade {
    pub fn step(&mut self, command: MindCommand) -> Result<MindCycleOutput, MindError>;
}
```

Deterministic sequence per step:

1. ingest command,
2. increment cycle and update `MindState`,
3. if command includes new goal and active goal exists:
   - compute safe point,
   - run preemption decision,
   - apply disposition via `GoalManager`,
4. obtain delegation plan (if needed) from `DelegationCoordinatorPort`,
5. run `NormativeEvaluator`,
6. build conflict cases and resolve via `ConflictResolver`,
7. call `MemoryPolicyPort` with evaluation report,
8. run `EvolutionDecider`,
9. emit typed `MindEvent` + `MindDecision`.

## 2) Command Handling Algorithm

### 2.1 `MindCommand::ProposeGoal(goal)`

1. register incoming goal,
2. if no active goal: activate incoming directly,
3. else run preemption subroutine,
4. enforce invariants,
5. continue to delegation/evaluation/conflict/evolution stages.

### 2.2 `MindCommand::ObserveSignal`

1. append observation-derived intent/evidence,
2. skip goal preemption stage,
3. run evaluation/conflict/evolution.

### 2.3 `MindCommand::SubmitDelegationResult`

1. append to recent delegation results buffer,
2. run evaluation/conflict/evolution.

### 2.4 `MindCommand::EvaluateNow`

1. no preemption,
2. evaluate current state,
3. resolve conflicts,
4. produce evolution decision.

## 3) Preemption Algorithm (Deterministic)

Inputs:

- active goal record,
- incoming goal,
- safe point,
- optional recent reliability judgments.

Rules (ordered):

1. If there is no active goal: activate incoming; no preemption decision emitted.
2. If safe point is not preemptable: disposition is `Continue`.
3. If merge compatibility check passes and incoming + active are same parent scope and level-distance <= 1: disposition `Merge`.
4. Else if incoming priority is strictly greater than active priority: disposition `Pause`.
5. Else if active has repeated failure signal (last N evaluations reliability `Fail`): disposition `Cancel`.
6. Else disposition `Continue`.

Tie-breakers:

- for equal priority and no other rule triggered, choose `Continue`.
- if merge candidate ordering is ambiguous, lower lexical `GoalId` becomes merge target; other becomes merged source.

## 4) Applying Preemption Disposition

### 4.1 `Pause`

1. pause active goal,
2. activate incoming goal,
3. emit `GoalPaused` and `GoalActivated`.

### 4.2 `Cancel`

1. cancel active goal,
2. activate incoming goal,
3. emit `GoalCancelled` and `GoalActivated`.

### 4.3 `Continue`

1. keep active goal,
2. incoming remains `Proposed` backlog,
3. emit `PreemptionDecided(Continue)`.

### 4.4 `Merge`

1. materialize merged goal deterministically,
2. mark source goal(s) as `Merged`,
3. activate merged goal,
4. emit `GoalMerged` + `GoalActivated`.

## 5) Conflict Resolution Algorithm

Input conflict cases are processed in stable order:

1. sort by case kind order:
   - helper-output,
   - evaluator-verdict,
   - merge-compatibility,
2. within same kind, sort by stable key (`intent_id`, `criterion`, `(active_goal_id,incoming_goal_id)`).

Per kind strategy:

1. Helper conflict
- choose highest confidence,
- tie-break by lexical `helper_id`.

2. Evaluator conflict
- choose most conservative verdict by rank:
  - `Fail` > `Borderline` > `Unknown` > `Pass`,
- tie-break by higher confidence, then lexical rationale hash.

3. Merge compatibility conflict
- deterministic compatibility predicate.
- output `MergeAllowed` or `MergeRejected`.

## 6) Evolution Trigger Algorithm (Proposal-only)

Decision rules:

1. Build failure signals from evaluation:
- any `Fail` in `GoalAlignment`,
- repeated `Fail` in `SubsystemReliability`,
- repeated `Fail` in `SignalFaithfulness`.

2. Trigger threshold:
- propose change only if failure pattern persists for at least `2` recent reports for same active goal.

3. Target mapping:
- reliability failures -> `Model` or `PerceptionPipeline`,
- faithfulness failures -> `PerceptionPipeline`,
- chronic recall/forget mismatch (from memory directives) -> `MemoryStructure`.

4. Action mapping:
- acute configuration drift -> `Reconfigure`,
- persistent behavior deficit with same config -> `Replace`,
- retraining candidate only when rationale explicitly includes data insufficiency.

5. If threshold not met: `NoChange`.

## 7) Memory Policy Invocation

After evaluation and conflict resolution:

1. call `MemoryPolicyPort::decide(state, report)`,
2. record resulting directive into event/decision trail,
3. do not persist externally in MVP.

This preserves memory-related decision trace without introducing storage coupling.

## 8) Complexity And Determinism Notes

1. goal lookup and state updates: `O(log n)` via `BTreeMap`.
2. conflict sort: `O(k log k)` for `k` conflict cases.
3. all random/time-dependent behavior is forbidden inside core policies.
4. checkpoint token generation must be deterministic in tests (injectable provider or fixed token policy).
