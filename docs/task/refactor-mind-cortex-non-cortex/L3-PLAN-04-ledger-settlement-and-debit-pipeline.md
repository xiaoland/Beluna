# L3-04 - Ledger Settlement And Debit Pipeline
- Task Name: `refactor-mind-cortex-non-cortex`
- Stage: `L3` detail: reservation, settlement, external debit pipeline
- Date: `2026-02-10`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Reservation State Machine

States:
1. `Open`
2. `Settled`
3. `Refunded`
4. `Expired`

Valid transitions:
1. `Open -> Settled`
2. `Open -> Refunded`
3. `Open -> Expired`

Invalid transitions:
1. any terminal -> any other terminal.

Idempotency rule:
1. replay of same terminal op with same `(reserve_entry_id, reference_id)` is no-op.
2. replay with different `reference_id` is error.

## 2) Reservation Timeout Clock

Clock contract:
1. timeout uses cycle clock only.
2. `expires_at_cycle = created_cycle + reservation_ttl_cycles`.
3. expiration evaluated during reconciliation by current cycle.

Determinism impact:
1. replay under same cycle sequence yields same expiry behavior.

## 3) Settlement Event Linkage

Required on spine settlement events:
1. `reserve_entry_id`
2. `action_id`
3. `cost_attribution_id`

Why:
1. auditability of reserve->event->ledger linkage.
2. straightforward idempotent settlement.

## 4) Reconciliation Ordering

Input: `SpineExecutionReport.events: Vec<OrderedSpineEvent>`.

Algorithm:
1. sort/verify monotonic `seq_no`.
2. process strictly in `seq_no` order.
3. ignore transport arrival order.

Mode semantics:
1. `BestEffortReplayable`
- execution may vary, order/replay cursor must still be stable enough for replay.
2. `SerializedDeterministic`
- same admitted batch/state => same ordered stream.

## 5) Cost Delta Settlement

For `ActionApplied`:
1. lookup reservation amount by `reserve_entry_id`.
2. compute `delta = actual_cost_micro - reserved_amount`.
3. apply `Adjustment` entry if delta != 0.
4. settle reservation.

For `ActionRejected`:
1. credit full reserved amount.
2. refund reservation.

For open expired reservation:
1. apply expiration terminal.
2. no second terminal operation allowed later.

## 6) External Debit Attribution Chain

Minimum matching key:
1. `cost_attribution_id` (required).

Optional consistency keys:
1. `action_id` if present.
2. `cycle_id` if present.

Apply rule:
1. if attribution unmatched -> ignore observation.
2. if matched and `reference_id` unseen -> apply debit.
3. else (seen) -> ignore.

## 7) AI Gateway Approximate Feed Integration

### 7.1 Forward path
1. Cortex creates `cost_attribution_id` on attempt.
2. Non-cortex copies into admitted action.
3. Spine path passes attribution to AI Gateway request context/metadata.

### 7.2 Return path
1. AI Gateway telemetry emits attribution on completion.
2. `AIGatewayApproxDebitSource` converts telemetry to `ExternalDebitObservation`.
3. Non-cortex applies matched approximate debit entry.

## 8) Versioned Policy Guard

Reservation/admission/debit interpretation always tied to non-cortex state versions:
1. `affordance_registry_version`
2. `cost_policy_version`
3. `admission_ruleset_version`

Record requirement:
1. ledger entry note or metadata should include active version tuple for audit.

## 9) Failure Cases And Handling

1. Missing reservation for settlement event:
- emit non-cortex reconciliation error and skip terminal transition.

2. Duplicate settlement with different `reference_id`:
- reject as invariant violation.

3. External observation with mismatched attribution:
- ignore (do not tax unrelated spend).

4. Overflow/underflow in micro-unit math:
- return typed non-cortex error and abort transition.

## 10) Acceptance Checks
1. no open reservation beyond bounded cycle window.
2. no double terminal transition.
3. no external unmatched debit entry applied.
4. deterministic replay yields same ledger end-state.

Status: `READY_FOR_L3_REVIEW`
