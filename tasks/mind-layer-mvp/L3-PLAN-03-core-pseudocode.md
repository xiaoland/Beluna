# L3-03 - Core Pseudocode

- Task Name: `mind-layer-mvp`
- Stage: `L3` detail: core logic pseudocode
- Date: 2026-02-09
- Status: `DRAFT_FOR_APPROVAL`

## 1) `MindFacade::step`

```text
fn step(command):
  state.cycle_id += 1
  output = MindCycleOutput { cycle_id, events: [], decisions: [] }

  // 1. ingest + state update (base)
  apply_command_base_effects(command, state)

  // 2. if command is ProposeGoal and active exists: preemption phase
  if command is ProposeGoal(new_goal) and state.active_goal_id is Some(active_id):
    safe_point = safe_point_policy.inspect(state, Some(active_id))
    preemption = preemption_decider.decide(ctx(state, active_goal, new_goal, safe_point))

    apply_preemption_decision(state, preemption, new_goal)
    output.decisions.push(MindDecision::Preemption(preemption))
    output.events.push(MindEvent::PreemptionDecided { disposition: preemption.disposition })

  // 3. delegation planning
  intents = delegation_port.plan(state, state.active_goal_id)
  if intents not empty:
    state.pending_intents.extend(intents)
    output.decisions.push(MindDecision::DelegationPlan(intents))

  // 4. evaluation
  report = evaluator.evaluate(state, command)
  state.recent_evaluations.push_back(report)
  output.decisions.push(MindDecision::Evaluation(report))
  output.events.push(MindEvent::EvaluationCompleted)

  // 5. conflict resolution
  conflict_cases = build_conflict_cases(state)
  resolutions = conflict_resolver.resolve(conflict_cases)
  apply_conflict_resolutions(state, resolutions)
  for r in resolutions:
    output.decisions.push(MindDecision::Conflict(r))
  if resolutions not empty:
    output.events.push(MindEvent::ConflictResolved)

  // 6. memory policy
  mem_directive = memory_policy.decide(state, report)
  record_memory_directive_in_state_or_event(state, mem_directive)

  // 7. evolution decision
  evolution = evolution_decider.decide(state, report)
  output.decisions.push(MindDecision::Evolution(evolution))
  output.events.push(MindEvent::EvolutionDecided)

  GoalManager::assert_invariants(state)
  return output
```

## 2) Base Command Effects

```text
apply_command_base_effects(command, state):
  match command:
    ProposeGoal(goal):
      GoalManager::register_goal(state, goal)
      if no active goal:
        GoalManager::activate_goal(state, goal.id)
        emit GoalActivated

    ObserveSignal(signal):
      append observation marker into state (as evidence context)

    SubmitDelegationResult(result):
      clamp result.confidence to [0,1]
      state.recent_delegation_results.push_back(result)

    EvaluateNow:
      no base mutation required
```

## 3) `GoalManager` Operations

```text
register_goal(state, goal):
  reject if duplicate goal id
  reject invalid parent relation for mid/low goal
  insert record status=Proposed

activate_goal(state, goal_id):
  reject if current active exists and != goal_id
  reject if target status in Completed/Cancelled/Merged
  set target status=Active
  set active_goal_id=Some(goal_id)

pause_active_goal(state):
  require active_goal_id exists
  set active record status=Paused
  clear active_goal_id

cancel_goal(state, goal_id):
  set status=Cancelled
  if active_goal_id == goal_id clear active

merge_goals(state, active_id, incoming_id, merged_goal):
  register merged_goal
  set both source goals status=Merged with merged_into=merged_goal.id
  set active_goal_id=Some(merged_goal.id)
  set merged goal status=Active
```

## 4) Conflict Case Builder

```text
build_conflict_cases(state):
  cases = []

  group delegation results by intent_id
  if any group has >1 distinct helper_id:
    add HelperOutputSameIntent case

  group latest judgments by criterion in current cycle scope
  if any criterion has >1 inconsistent verdict:
    add EvaluatorVerdictSameCriterion case

  if last preemption disposition is Merge and compatibility uncertain:
    add MergeCompatibility case

  return cases sorted by deterministic key
```

## 5) Memory Directive Recording

```text
record_memory_directive_in_state_or_event(state, directive):
  // MVP: advisory-only, no persistence
  // store as cycle-local annotation to preserve audit trail
  state.last_memory_directive = Some(directive)
```

## 6) Error Surface

```text
MindErrorKind:
  InvalidRequest
  InvariantViolation
  PolicyViolation
  ConflictResolutionError
  Internal
```

All policy/port failures map to typed `MindError` without transport-specific leakage.

Status: `READY_FOR_L3_REVIEW`
