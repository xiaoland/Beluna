# L2 Plan - Refactor Mind into Cortex + Non-Cortex (Low-Level Design)
- Task Name: `refactor-mind-cortex-non-cortex`
- Stage: `L2` (low-level design)
- Date: `2026-02-10`
- Status: `DRAFT_FOR_APPROVAL`

This L2 is split into focused files to keep interfaces, algorithms, and tests reviewable.

## L2 File Index
1. `/Users/lanzhijiang/Development/Beluna/docs/task/refactor-mind-cortex-non-cortex/L2-PLAN-01-module-and-boundary-map.md`
- source/test/doc file map
- dependency and ownership boundaries

2. `/Users/lanzhijiang/Development/Beluna/docs/task/refactor-mind-cortex-non-cortex/L2-PLAN-02a-goal-commitment-and-scheduling.md`
- goal semantic identity model, commitment model, and cycle-local scheduling context

3. `/Users/lanzhijiang/Development/Beluna/docs/task/refactor-mind-cortex-non-cortex/L2-PLAN-02b-effectuation-ledger-and-invariants.md`
- effectuation/admitted-action model, survival ledger, continuity invariants

4. `/Users/lanzhijiang/Development/Beluna/docs/task/refactor-mind-cortex-non-cortex/L2-PLAN-03-ports-and-policy-contracts.md`
- trait interfaces for cortex/non-cortex/spine integration
- policy contracts and mechanical enforcement rules

5. `/Users/lanzhijiang/Development/Beluna/docs/task/refactor-mind-cortex-non-cortex/L2-PLAN-04-deterministic-loop-and-algorithms.md`
- deterministic end-to-end cycle algorithm
- admission, degradation, ledger, and spine flow pseudo-code

6. `/Users/lanzhijiang/Development/Beluna/docs/task/refactor-mind-cortex-non-cortex/L2-PLAN-05-test-contract-and-doc-plan.md`
- BDD contract plan
- test matrix and documentation migration plan

## L2 Objective
Define exact interfaces, data structures, and algorithms for a canonical split where:
1. Cortex emits non-binding `IntentAttempt[]`.
2. Non-cortex performs mechanical admission/effectuation decisions.
3. Spine executes only `AdmittedAction[]` (never attempts).
4. Continuity/survival is preserved in non-cortex through a global survival budget ledger.
5. Goal semantic identity is separated from commitment lifecycle and cycle-local scheduling pressure.
6. Cost attribution is carried end-to-end (`IntentAttempt -> AdmittedAction -> Spine/Gateway telemetry`) for safe external debit matching.
7. Spine execution semantics are explicit (`BestEffortReplayable` or `SerializedDeterministic`) with ordered-event requirements.

## L2 Completion Gate
L2 is complete when:
1. the `IntentAttempt -> admission -> AdmittedAction` boundary is mechanically enforceable,
2. non-cortex non-interpretation constraints are encoded in contracts/tests,
3. global survival ledger design (including AI Gateway approximate debit feed) is unambiguous,
4. L3 can execute directly without redefining architecture.

Status: `READY_FOR_REVIEW`
