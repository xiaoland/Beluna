# L3-01 - Workstreams And Sequence
- Task Name: `refactor-mind-cortex-non-cortex`
- Stage: `L3` detail: execution sequence
- Date: `2026-02-10`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Execution Principles
1. Canonical surfaces are `cortex`, `non_cortex`, `spine`; `mind` is removed.
2. `IntentAttempt` remains non-binding; Spine executes only `AdmittedAction`.
3. Admission purity is strict: no wall-clock/random/unordered iteration.
4. Settlement consistency is strict: each reservation terminalizes once (`Settled|Refunded|Expired`) within bounded cycles.
5. External debits require attribution-chain match (`cost_attribution_id`).

## 2) Ordered Workstreams

### Workstream A - Module Skeleton And Public Surface Cutover
Scope:
1. create `core/src/cortex/*`, `core/src/non_cortex/*`, `core/src/spine/*`.
2. expose modules from `core/src/lib.rs`.
3. remove `core/src/mind/*` and `pub mod mind`.

Exit criteria:
1. project compiles with new module stubs.
2. no references to removed `mind` symbols remain.

### Workstream B - Domain Types + Deterministic ID Derivation
Scope:
1. implement goal/commitment split and typed metadata/provenance.
2. implement `IntentAttempt` and deterministic `AttemptId` derivation.
3. implement effectuation/admission types and deterministic `ActionId` derivation.

Exit criteria:
1. type model compiles.
2. deterministic ID tests pass.

### Workstream C - Non-Cortex Admission Engine
Scope:
1. implement hard constraint, economic estimation, and cost admission policy hooks.
2. implement deterministic degradation search (rank + caps + stop policy).
3. produce `AdmissionBatchResult` including `AdmissionReport`.

Exit criteria:
1. denied/admitted/degraded outcomes are deterministic.
2. admission report includes all attempts.

### Workstream D - Survival Ledger + Reservation Lifecycle
Scope:
1. implement global ledger and reservation creation.
2. implement `settle|refund|expire` with strict idempotency.
3. implement cycle-clock expiry (`expires_at_cycle`).

Exit criteria:
1. settlement consistency tests pass.
2. no reservation leak in bounded-cycle tests.

### Workstream E - Spine Contracts + Ordered Event Handling
Scope:
1. implement spine types/ports/no-op adapter.
2. enforce only `AdmittedActionBatch` dispatch.
3. process ordered `seq_no` events for reconciliation.

Exit criteria:
1. spine compile-time boundary checks pass.
2. best-effort replayable and serialized deterministic semantics represented in report mode.

### Workstream F - Cortex/Non-Cortex Integration Facades
Scope:
1. implement cortex step with scheduling recomputation.
2. invoke non-cortex kernel with attempts and spine port.
3. return outcomes/admission report/feedback to cortex loop.

Exit criteria:
1. end-to-end cycle works with no-op spine.
2. denied outcomes are surfaced to cortex each cycle.

### Workstream G - AI Gateway Approx Debit Adapter
Scope:
1. extend gateway request/telemetry to carry `cost_attribution_id`.
2. implement `AIGatewayApproxDebitSource` ingestion adapter.
3. enforce attribution matching + reference dedupe.

Exit criteria:
1. unmatched attribution is ignored.
2. matched approximate debit updates ledger exactly once.

### Workstream H - Tests, Docs, Result
Scope:
1. replace `core/tests/mind/*` with new suites.
2. migrate feature/module/contract docs to cortex/non-cortex/spine.
3. write task result summary file.

Exit criteria:
1. targeted tests green.
2. docs indexes updated and coherent.

## 3) Dependency Graph
1. A -> B
2. B -> C
3. C -> D
4. C + D -> E
5. B + C + D + E -> F
6. F -> G
7. G -> H

## 4) Stop/Go Checkpoints
1. After A: verify no lingering `mind` imports.
2. After B: verify deterministic ID derivation reproducibility.
3. After D: verify reservation lifecycle terminality/invariant.
4. After F: verify admitted-only dispatch and denied feedback return.
5. After G: verify attribution-chain debit matching.

## 5) Out Of Scope (This Implementation)
1. concrete body endpoint runtime integration beyond spine contracts/no-op.
2. full-fidelity monetary pricing model (approx debit remains v1).
3. socket protocol exposure of cortex/non-cortex/spine commands.

Status: `READY_FOR_L3_REVIEW`
