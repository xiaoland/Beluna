# L2 Plan 05 - Test, Contract, And Documentation Plan

- Task Name: `mind-layer-mvp`
- Stage: `L2` / Part 05
- Date: 2026-02-08

## 1) BDD Contract Files To Add

```text
docs/contracts/mind/
├── README.md
├── goal-management.md
├── preemption.md
├── evaluation.md
├── delegation-and-conflict.md
├── evolution-trigger.md
└── facade-loop.md
```

Contract focus:

1. `goal-management.md`
- single-active-goal invariant
- valid/invalid status transitions

2. `preemption.md`
- disposition set is closed (`Pause|Cancel|Continue|Merge`)
- safe point and checkpoint token constraints

3. `evaluation.md`
- normative criteria and judgment output schema

4. `delegation-and-conflict.md`
- trait-based delegation boundary
- scoped conflict ownership and deterministic resolution

5. `evolution-trigger.md`
- proposal-only evolution decisions
- threshold behavior and target/action mapping

6. `facade-loop.md`
- deterministic seven-step loop ordering
- event/decision emission guarantees

## 2) Test Plan (`tests/mind/*`)

### 2.1 Goal manager tests

1. `given_no_active_goal_when_activate_then_goal_becomes_active`
2. `given_active_goal_when_activate_other_without_preemption_then_rejected`
3. `given_merged_goal_when_reactivate_then_rejected`
4. `given_invalid_parent_goal_when_register_then_invalid_request`

### 2.2 Preemption tests

1. `given_non_preemptable_safe_point_when_new_goal_then_continue`
2. `given_higher_priority_and_preemptable_when_new_goal_then_pause`
3. `given_reliability_failures_when_new_goal_then_cancel`
4. `given_merge_compatible_goals_when_new_goal_then_merge`
5. `given_checkpoint_without_preemptable_when_decide_then_invalid_request`

### 2.3 Evaluator tests

1. `given_alignment_evidence_when_evaluate_then_alignment_judgment_emitted`
2. `given_missing_evidence_when_evaluate_then_unknown_verdict_emitted`
3. `given_non_pass_verdict_when_evaluate_then_rationale_is_non_empty`

### 2.4 Conflict resolver tests

1. `given_helper_conflict_when_resolve_then_highest_confidence_selected`
2. `given_helper_confidence_tie_when_resolve_then_lexical_helper_id_selected`
3. `given_evaluator_conflict_when_resolve_then_most_conservative_verdict_selected`
4. `given_merge_conflict_when_resolve_then_deterministic_allow_or_reject`

### 2.5 Evolution tests

1. `given_single_failure_when_decide_then_no_change`
2. `given_repeated_reliability_failures_when_decide_then_change_proposal`
3. `given_faithfulness_failures_when_decide_then_perception_target_proposed`
4. `given_low_confidence_when_decide_then_no_change`

### 2.6 Facade loop tests

1. `given_same_input_and_state_when_step_twice_then_outputs_are_identical`
2. `given_new_goal_with_active_goal_when_step_then_preemption_before_delegation`
3. `given_evaluation_report_when_step_then_conflict_resolution_before_evolution`
4. `given_noop_ports_when_step_then_loop_completes_without_external_side_effect`

## 3) Documentation Files To Add/Update

1. New feature package:

```text
docs/features/mind/
├── README.md
├── PRD.md
├── HLD.md
└── LLD.md
```

2. New module package:

```text
docs/modules/mind/
├── README.md
├── purpose.md
├── architecture.md
├── execution-flow.md
└── policies.md
```

3. Update indexes:

- `docs/features/README.md`
- `docs/modules/README.md`
- `docs/contracts/README.md`
- `docs/product/overview.md` (capability + limitation phrasing)
- `docs/product/glossary.md` (Mind-specific terms if missing)

## 4) L2 -> L3 Handoff Checklist

Before L3 drafting:

1. all core types and trait signatures are accepted,
2. disposition semantics (`pause/cancel/continue/merge`) are locked,
3. safe point/checkpoint placement is locked,
4. conflict ownership is locked,
5. deterministic loop order is locked,
6. evolution proposal-only rule is locked.

Status: `READY_FOR_L3_PLANNING_AFTER_APPROVAL`
