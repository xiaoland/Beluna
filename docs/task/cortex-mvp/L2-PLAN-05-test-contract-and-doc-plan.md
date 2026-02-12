# L2-05 - Test Contract And Doc Plan
- Task Name: `cortex-mvp`
- Stage: `L2` detailed file
- Date: `2026-02-11`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Contract To Test Matrix
### A. Reactor progression and always-on semantics
1. `given_open_inbox_when_events_arrive_then_reactor_advances_one_cycle_per_event`
2. `given_closed_inbox_when_run_then_reactor_stops_gracefully`
3. `given_bounded_channel_full_when_send_then_backpressure_is_mechanical`

### B. Call-budget and boundedness guarantees
4. `given_reaction_cycle_when_running_then_primary_call_count_is_exactly_one`
5. `given_extractor_and_repair_path_when_running_then_subcall_count_never_exceeds_limit`
6. `given_empty_first_clamp_and_repair_used_when_cycle_finishes_then_no_second_repair_occurs`
7. `given_limits_violation_when_cycle_executes_then_result_is_noop`

### C. Contract-field and world-relativity guarantees
8. `given_non_noop_output_when_emitted_then_every_attempt_has_attempt_id_and_based_on`
9. `given_feedback_signal_when_ingested_then_attempt_id_correlation_is_preserved`
10. `given_based_on_ids_when_clamped_then_unknown_sense_ids_are_rejected`

### D. Routing and clamp behavior
11. `given_unknown_affordance_when_clamped_then_attempt_is_dropped`
12. `given_unsupported_capability_handle_when_clamped_then_attempt_is_dropped`
13. `given_payload_schema_violation_when_clamped_then_attempt_is_dropped`
14. `given_more_than_max_attempts_when_clamped_then_stable_prefix_is_emitted`

### E. Repair and fallback behavior
15. `given_first_clamp_empty_when_filler_repairs_then_second_clamp_emits_attempts`
16. `given_repair_failure_when_cycle_finishes_then_noop_is_emitted`
17. `given_second_clamp_empty_when_cycle_finishes_then_noop_is_emitted`

### F. Statelessness guarantees
18. `given_two_identical_inputs_when_reacted_then_outputs_match_without_internal_persistence_dependency`
19. `given_prior_cycle_goal_material_not_present_when_next_cycle_reacts_then_cortex_does_not_read_hidden_state`

### G. AI gateway adapter and mock strategy
20. `given_runtime_adapter_when_primary_called_then_ai_gateway_infer_once_receives_expected_limits`
21. `given_mock_ports_in_tests_when_reactor_runs_then_no_network_io_occurs`
22. `given_sub_backend_without_tool_capability_when_extractor_called_then_cycle_falls_back_to_noop`

## 2) Test File Plan
Planned file directions:
1. `core/tests/cortex/reactor.rs` (new)
- always-on loop and call-budget contracts.
2. `core/tests/cortex/clamp.rs` (new)
- routing/schema/cap/payload/based_on rules.
3. `core/tests/cortex/ai_gateway_adapter.rs` (new)
- runtime adapter request construction + capability mismatch handling.
4. `core/tests/cortex_continuity_flow.rs` (update)
- preserve attempt correlation through admission feedback path.

Potential retirements:
1. old `core/tests/cortex/planner.rs` and commitment-centric tests once step API is removed.

## 3) Test Execution Plan
Primary commands (post-implementation):

```bash
cd /Users/lanzhijiang/Development/Beluna/core
cargo test cortex:: -- --nocapture
cargo test cortex_continuity_flow -- --nocapture
cargo test
```

Target checks:
1. no flaky async reactor tests.
2. no external network dependency in test runs.
3. boundedness limits asserted directly by call counters and clamp outputs.

## 4) Documentation Update Plan
Update/align docs after implementation:
1. `docs/features/cortex/PRD.md`
- replace step API language with reactor-only semantics.
2. `docs/features/cortex/HLD.md`
- update boundary to `ReactionInput -> ReactionResult`.
3. `docs/features/cortex/LLD.md`
- add IR/sub-LLM/clamp pipeline and one-repair/noop rules.
4. `docs/contracts/cortex/README.md`
- add `attempt_id` + `based_on` and feedback-correlation requirements.
5. `docs/overview.md`
- reflect `Sense + EnvSnapshot stream` wording and always-on progression semantics.
6. `core/AGENTS.md`
- refresh live capabilities and current limitations for new Cortex runtime behavior.

## 5) Regression Risk Checklist
1. Admission and continuity still consume `IntentAttempt[]` without semantic interpretation.
2. Existing ledger terminality invariants are untouched by cortex cutover.
3. Server protocol changes do not break `exit` handling.
4. AI gateway runtime path is optional in tests via mock ports.

## 6) Acceptance Gate For L3
L3 can start only if:
1. tests listed above are accepted as minimum contract coverage,
2. doc updates are accepted as part of implementation done criteria,
3. old step/planner contracts are confirmed removable.

## 7) L2-05 Exit Criteria
This file is complete when:
1. contract assertions map directly to named tests,
2. execution commands and regression gates are explicit,
3. documentation alignment tasks are enumerated and scoped.
