# L2 Plan 04 - Deterministic Loop And Algorithms
- Task Name: `refactor-mind-cortex-non-cortex`
- Stage: `L2` / Part 04
- Date: `2026-02-10`
## 1) End-to-End Deterministic Cycle
```text
1) Cortex ingests command and updates cortex state.
2) Cortex emits IntentAttempt[] (non-binding).
3) Non-cortex resolves attempts:
   - hard constraints
   - economic affordability
   - optional degradation
   - admitted actions + effectuation outcomes + admission report + ledger entries
4) Non-cortex sends only AdmittedAction[] to Spine.
5) Spine returns ordered execution/feedback events.
6) Non-cortex reconciles ledger using spine events and external debit observations.
7) Cortex consumes `AdmissionReport` (including denied outcomes) and spine feedback for next cycle.
```
## 2) Cortex Step Pseudo-code
```rust
fn cortex_step(state: &mut CortexState, cmd: CortexCommand) -> Result<Vec<IntentAttempt>, CortexError> {
    state.cycle_id = state.cycle_id.saturating_add(1);
    apply_goal_and_context_updates(state, &cmd)?;
    let scheduling = recompute_scheduling_contexts(state, state.cycle_id)?;
    let mut attempts = planner.plan_attempts(state, &scheduling, &cmd)?;
    attempts.sort_by(|a, b| a.attempt_id.cmp(&b.attempt_id));
    reject_duplicate_attempt_ids(&attempts)?;
    Ok(attempts)
}
```
Rules:
1. output attempts are proposals only.
2. attempts are sorted and deduplicated deterministically.
3. priority is recomputed each cycle from `SchedulingContext` and never read from `Goal` identity.
## 3) Non-Cortex Admission Algorithm
For each attempt in sorted order:
```text
A) Resolve affordance profile by affordance_key.
   - if missing => DeniedHard(code="unknown_affordance")
B) Run hard constraints.
   - if failed => DeniedHard(code=<hard rule code>)
C) Estimate economic cost.
D) Compute affordability snapshot and reserve amount via CostAdmissionPolicy.
   - if `snapshot.available_survival_micro >= snapshot.reserve_survival_micro`
     and ledger can reserve that amount:
   - create reservation with TTL in cycles
   - materialize AdmittedAction
   - outcome = Admitted{degraded=false}
E) Else try degradation candidates (deterministic ranked search).
   - rank candidates by configured tuple:
     - prefer-less-loss mode:
       `(capability_loss_score, estimated_survival_micro, profile_id)`
     - cheapest-first mode:
       `(estimated_survival_micro, capability_loss_score, profile_id)`
   - enforce search caps:
     - max variants
     - max degradation depth
   - for each ranked candidate (within caps):
     1. apply candidate patch/resource downgrade to a virtual attempt
     2. re-run hard constraints
     3. re-estimate cost
     4. re-run CostAdmissionPolicy affordability + reserve amount
     5. if affordable => reserve + materialize AdmittedAction(degraded)
        outcome = Admitted{degraded=true}, stop search
F) If no candidate affordable:
   - DeniedEconomic(code="insufficient_survival_budget")
```
Determinism rules:
1. candidate order sorted by configured deterministic rank tuple.
2. no random and no wall-clock dependence.
## 4) Admitted Action Materialization
```rust
fn materialize_admitted(
    cycle_id: CycleId,
    attempt: &IntentAttempt,
    degradation: Option<DegradationPlan>,
    reserved_cost: CostVector,
    ledger_entry_id: LedgerEntryId,
) -> (AdmittedAction, EffectuationOutcome)
```
Generation rules:
1. `action_id` uses fixed canonical derivation (L2-02b), never random/time-based.
2. `admission_proof` is attached internally by non-cortex.
3. outcome references `admitted_action_id` and `ledger_entry_id`.
4. `cost_attribution_id` is copied from attempt into admitted action.
## 5) Spine Dispatch Algorithm
```rust
fn dispatch_to_spine(
    cycle_id: CycleId,
    admitted_actions: Vec<AdmittedAction>,
    spine: &dyn SpinePort,
) -> Result<SpineExecutionReport, NonCortexError>
```
Flow:
1. build `AdmittedActionBatch`.
2. call `spine.execute_admitted(&batch)`.
3. return report as-is to non-cortex reconciliation step.
Guarantee:
- no denied attempt is forwarded to spine.
- admitted actions carry `cost_attribution_id`.
## 6) Ledger Reconciliation Algorithm
Inputs:
1. reserved admission entries from this cycle.
2. spine execution report (actual costs/rejections).
3. external debit observations (AI Gateway approximate feed).
Pseudo-code:
```text
0) Sort spine events by `seq_no` and process in ascending order.
1) For each SpineEvent::ActionApplied with actual_cost_micro:
   - lookup reservation by `reserve_entry_id` from event
   - compute delta = actual - reserved
   - apply Adjustment entry (positive or negative) with source=SpineSettlement
   - settle reservation for same `reserve_entry_id` (terminal=Settled)
2) For each SpineEvent::ActionRejected:
   - lookup reservation by `reserve_entry_id` from event
   - apply Credit for previously reserved amount (source=SpineSettlement)
   - refund same `reserve_entry_id` (terminal=Refunded)
3) For still-open reservations where `current_cycle >= expires_at_cycle`:
   - expire reservation (terminal=Expired)
4) Drain external debit observations:
   - require `cost_attribution_id` match to known admitted action/attempt chain
   - if `action_id` and/or `cycle_id` are present, require consistency with that chain
   - skip if reference_id already seen
   - apply Debit entry with source and accuracy from observation
   - mark reference_id seen
```
Reservation strictness:
- `settle|refund|expire` are mutually exclusive terminal transitions per reservation.
- repeated settlement with same `(reserve_entry_id, reference_id)` is idempotent no-op.
## 7) AI Gateway Approximate Debit Mapping
Initial mapping policy (v1):
1. observation source: `GatewayTelemetryEvent::RequestCompleted`.
2. reference ID: `ai_gateway:<request_id>`.
3. attribution: telemetry must include `cost_attribution_id`.
4. amount estimation:
- if `usage.total_tokens` exists: `amount_micro = total_tokens as i128 * TOKEN_MICRO_RATE`.
- else if `usage.output_tokens` exists: use output tokens.
- else fallback fixed debit for completed request.
5. accuracy: always `Approximate`.
6. unmatched attribution IDs are ignored (no unrelated taxation).
Rationale:
- feeds global survival ledger early without blocking on perfect accounting.
## 8) Admission Purity And Non-Interpretation
Admission decisions depend only on:
1. `NonCortexState`
2. attempt (`affordance_key`, `capability_handle`, `requested_resources`, `normalized_payload`)
3. deterministic registries/policies
4. deterministic version tuple in state:
- `affordance_registry_version`
- `cost_policy_version`
- `admission_ruleset_version`
Forbidden inputs:
1. wall-clock
2. randomness
3. unordered iteration artifacts
Non-interpretation constraint:
- no branching on semantic narrative fields.
Admission branch inputs are restricted to:
1. `affordance_key`
2. `capability_handle`
3. resource requests
4. affordance profile constraints/cost tables
5. current ledger and continuity state
Explicitly ignored for branching:
1. goal title or narrative text
2. semantic metadata labels ("defection", "compliance", etc.)
3. natural-language content in payload fields not bound to affordance schema keys
## 9) Spine Semantics Contract (Explicit)
Two valid spine execution semantics:
1. `BestEffortReplayable`
- tool retries/backoff/network variance are allowed.
- emitted events must be totally ordered by `seq_no`.
- replay must be possible via `replay_cursor`.
2. `SerializedDeterministic`
- execution is serialized with deterministic ordering guarantees.
- for same admitted batch + same state, ordered event stream is identical.
Ledger reconciliation requirement:
- always consume events in `seq_no` order, never arrival order.
## 10) Admission Report Returned To Cortex
Every cycle, non-cortex returns:
1. per-attempt disposition:
- `Admitted{degraded=false}`
- `Admitted{degraded=true}`
- `DeniedHard(code)`
- `DeniedEconomic(code)`
2. schema-limited details:
- hard rule code (for hard denial)
- affordability snapshot numbers (for admitted/economic denial)
3. ledger deltas:
- reserve entry ids and reserved amounts.
This forms a stable learning signal without moral narration.
## 11) Failure Handling Rules
1. Unknown affordance/capability => `DeniedHard`.
2. Invalid attempt shape => `DeniedHard(code="invalid_attempt_shape")`.
3. Spine call failure => no silent drop:
- emit non-cortex error event,
- keep continuity and ledger state consistent (no double debit).
## 12) Complexity Notes
1. attempt admission: `O(n * (h + d))`
- `n` attempts, `h` hard checks, `d` degradation candidates.
2. ledger operations: `O(log m)` index/set costs (`m` entry count).
3. deterministic sort cost: `O(n log n)`.
## 13) End-to-End Contract Form (Locked)
```text
Cortex -> IntentAttempt[] -> Non-cortex admits/denies -> Spine executes AdmittedAction[]
```
