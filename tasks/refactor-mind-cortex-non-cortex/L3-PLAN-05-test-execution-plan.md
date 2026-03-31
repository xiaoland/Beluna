# L3-05 - Test Execution Plan
- Task Name: `refactor-mind-cortex-non-cortex`
- Stage: `L3` detail: test plan and execution
- Date: `2026-02-10`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Contract To Test Mapping

### 1.1 Cortex contracts
1. goal/commitment split, failed status, supersession relation.
2. deterministic scheduling and attempt derivation.
3. deterministic `AttemptId` generation.

### 1.2 Non-cortex contracts
1. admission purity and non-interpretation.
2. disposition completeness (`Admitted{degraded}|DeniedHard{code}|DeniedEconomic{code}`).
3. deterministic degradation ranking and bounded search.
4. reservation lifecycle terminality/idempotency.
5. attribution-matched external debit only.

### 1.3 Spine contracts
1. admitted-action-only interface.
2. ordered replayable event stream semantics.
3. reserve-entry-linked settlement events.

## 2) Test Suites To Implement

### 2.1 `core/tests/cortex/*`
1. deterministic planner output.
2. commitment state transitions.
3. metadata type/provenance preservation.
4. deterministic attempt-id derivation.

### 2.2 `core/tests/non_cortex/*`
1. hard denial and economic denial with codes.
2. admitted without degradation and admitted with degradation.
3. admission report includes all attempts.
4. admission purity against semantic-field perturbation.
5. degradation tie-break and search cap behavior.
6. reservation settle/refund/expire terminal strictness.
7. reservation expiry at bounded cycles.
8. idempotent settlement with same reference.
9. unmatched attribution ignored.
10. duplicate external reference ignored.

### 2.3 `core/tests/spine/*`
1. admitted-only dispatch contract.
2. best-effort replayable mode event ordering/replay cursor.
3. serialized deterministic mode reproducibility.
4. settlement events include `reserve_entry_id`.

### 2.4 Integration (`core/tests/cortex_non_cortex_flow.rs`)
1. only admitted actions reach spine.
2. denied outcomes return via admission report.
3. attribution chain preserved through cycle.
4. ordered-event reconciliation independent of arrival order.
5. cortex replacement keeps non-cortex continuity and ledger state.

## 3) Execution Order
1. run targeted unit tests for new modules first.
2. run integration flow tests.
3. run AI Gateway-related debit-source tests.
4. run full `cargo test`.

## 4) Command Plan
1. `cd /Users/lanzhijiang/Development/Beluna/core`
2. `cargo test cortex::`
3. `cargo test non_cortex::`
4. `cargo test spine::`
5. `cargo test cortex_non_cortex_flow`
6. `cargo test ai_gateway::`
7. `cargo test`

Note:
1. exact filter selectors may vary by module naming; adjust to concrete test module names once created.

## 5) Determinism Verification Tactics
1. duplicate-run equality assertions for admission results.
2. stable sorting assertions for attempts and degradation candidates.
3. deterministic id derivation golden vectors.
4. replay reconciliation assertions by `seq_no`.

## 6) Acceptance Criteria
1. new cortex/non-cortex/spine tests pass.
2. no leftover mind tests or compile references.
3. full test suite passes.
4. all L2 invariants have at least one test assertion.

## 7) Regression Guardrails
1. no direct protocol/server dependency in new modules.
2. no runtime randomness/time usage in admission path.
3. no unmatched external debit application.

Status: `READY_FOR_L3_REVIEW`
