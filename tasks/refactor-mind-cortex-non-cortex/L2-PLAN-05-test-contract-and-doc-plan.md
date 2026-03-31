# L2 Plan 05 - Test, Contract, And Documentation Plan
- Task Name: `refactor-mind-cortex-non-cortex`
- Stage: `L2` / Part 05
- Date: `2026-02-10`

## 1) Contract Documentation Plan

### 1.1 Cortex contracts

```text
docs/contracts/cortex/
├── README.md
├── goal-lifecycle.md
├── attempt-generation.md
└── deterministic-loop.md
```

### 1.2 Non-cortex contracts

```text
docs/contracts/non-cortex/
├── README.md
├── admission-and-effectuation.md
├── non-interpretation.md
├── survival-ledger.md
└── external-debit-feed.md
```

### 1.3 Spine contracts (scope-limited)

```text
docs/contracts/spine/
├── README.md
├── admitted-action-dispatch.md
├── feedback-events.md
└── execution-semantics.md
```

## 2) Core Test Matrix

### 2.1 Cortex tests (`core/tests/cortex/*`)

1. `given_same_state_and_command_when_plan_attempts_then_output_is_identical`
2. `given_goal_and_commitment_split_when_step_then_single_active_commitment_invariant_holds`
3. `given_invalid_goal_parent_when_register_then_invalid_request`
4. `given_attempt_id_collision_when_plan_then_rejected`
5. `given_supersession_when_transition_then_relationship_recorded_without_merged_status`
6. `given_commitment_failure_when_transition_then_status_is_failed_with_failure_code`
7. `given_goal_metadata_entries_when_register_then_typed_values_and_provenance_are_preserved`
8. `given_same_derivation_input_when_generate_attempt_id_then_attempt_id_is_identical`

### 2.2 Non-cortex tests (`core/tests/non_cortex/*`)

1. `given_unknown_affordance_when_resolve_then_denied_hard`
2. `given_hard_constraint_failure_when_resolve_then_denied_hard`
3. `given_affordable_attempt_when_resolve_then_admitted_and_debited`
4. `given_insufficient_budget_with_degrade_path_when_resolve_then_admitted_with_degraded_true`
5. `given_insufficient_budget_without_degrade_path_when_resolve_then_denied_economic`
6. `given_same_attempt_except_semantic_text_when_resolve_then_same_outcome`
7. `given_spine_rejection_when_reconcile_then_reserved_cost_is_credited`
8. `given_duplicate_external_reference_when_ingest_then_second_is_ignored`
9. `given_external_ai_gateway_debit_when_ingest_then_accuracy_is_approximate`
10. `given_floor_budget_boundary_when_admit_then_balance_never_below_floor`
11. `given_reservation_terminal_transition_when_settle_then_refund_or_expire_is_rejected`
12. `given_unmatched_cost_attribution_when_external_debit_ingest_then_observation_is_ignored`
13. `given_cycle_process_when_resolve_then_admission_report_includes_all_denied_and_admitted_outcomes`
14. `given_degradation_candidates_tied_when_resolve_then_rank_tuple_breaks_tie_deterministically`
15. `given_degradation_search_caps_when_resolve_then_search_stops_at_max_depth_or_variants`
16. `given_denied_outcome_when_reported_then_result_fields_contain_only_schema_keys`
17. `given_same_action_derivation_input_when_materialize_then_action_id_is_identical`
18. `given_reservation_open_when_ttl_cycles_elapsed_then_reservation_expires`
19. `given_repeated_settlement_reference_when_settle_then_operation_is_idempotent`

### 2.3 Spine tests (`core/tests/spine/*`)

1. `given_admitted_batch_when_execute_then_batch_completed_event_present`
2. `given_empty_batch_when_execute_then_no_failure`
3. `given_same_batch_when_execute_twice_then_event_order_is_stable`
4. `given_best_effort_mode_when_execute_then_seq_no_is_total_and_replay_cursor_present`
5. `given_serialized_deterministic_mode_when_execute_then_order_is_reproducible`
6. `given_settlement_events_when_emitted_then_each_event_carries_reserve_entry_id`

### 2.4 Integration tests

File: `core/tests/cortex_non_cortex_flow.rs`

1. `given_attempts_when_process_then_only_admitted_actions_reach_spine`
2. `given_denied_attempts_when_process_then_spine_receives_empty_batch`
3. `given_cortex_replacement_when_next_cycle_then_non_cortex_continuity_persists`
4. `given_same_initial_state_and_inputs_when_cycle_then_outputs_are_identical`
5. `given_cost_attribution_chain_when_cycle_completes_then_attempt_action_and_gateway_debit_match`
6. `given_spine_events_out_of_arrival_order_when_reconcile_then_seq_order_processing_is_used`

## 3) Mechanical Enforcement Verification

Compile-time/API checks:
1. `SpinePort` exposes only `execute_admitted(AdmittedActionBatch)`.
2. there is no `execute_attempts(IntentAttempt[])` API.
3. `AdmittedAction` constructor is non-public outside non-cortex resolver.

Behavior checks:
1. denied outcomes never appear in dispatched spine batch.
2. semantic metadata mutation does not alter non-cortex decisions unless affordance/economic keys change.
3. `AdmissionReport` is returned to cortex every cycle and contains all attempt outcomes.

## 4) Documentation Migration Plan

### 4.1 Feature docs

```text
docs/features/cortex/{README,PRD,HLD,LLD}.md
docs/features/non-cortex/{README,PRD,HLD,LLD}.md
docs/features/spine/{README,PRD,HLD,LLD}.md
```

### 4.2 Module docs

```text
docs/modules/cortex/{README,purpose,architecture,execution-flow,policies}.md
docs/modules/non-cortex/{README,purpose,architecture,execution-flow,policies}.md
docs/modules/spine/{README,purpose,architecture,execution-flow,policies}.md
```

### 4.3 Index and glossary updates

1. update `/Users/lanzhijiang/Development/Beluna/docs/features/README.md`.
2. update `/Users/lanzhijiang/Development/Beluna/docs/modules/README.md`.
3. update `/Users/lanzhijiang/Development/Beluna/docs/contracts/README.md`.
4. update `/Users/lanzhijiang/Development/Beluna/docs/product/overview.md`.
5. update `/Users/lanzhijiang/Development/Beluna/docs/product/glossary.md`.

### 4.4 Legacy docs policy

1. deprecate `docs/features/mind/*`, `docs/modules/mind/*`, `docs/contracts/mind/*`.
2. either delete or mark as superseded by cortex/non-cortex/spine docs in the same PR.

## 5) L2 -> L3 Handoff Checklist

Before L3:
1. module/file map is approved.
2. domain types and invariants are approved.
3. admission/spine/ledger interfaces are approved.
4. deterministic algorithm order is approved.
5. test and documentation migration plan is approved.

Status: `READY_FOR_L3_PLANNING_AFTER_APPROVAL`
