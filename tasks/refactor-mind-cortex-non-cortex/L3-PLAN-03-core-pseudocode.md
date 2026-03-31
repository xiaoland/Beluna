# L3-03 - Core Pseudocode
- Task Name: `refactor-mind-cortex-non-cortex`
- Stage: `L3` detail: core execution logic
- Date: `2026-02-10`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Top-Level Cycle

```text
fn run_cycle(cortex, non_cortex, spine, cmd):
  attempts = cortex.step(cmd)                        // non-binding proposals
  nc_output = non_cortex.process_attempts(attempts, spine)
  cortex.observe_admission_and_feedback(
    nc_output.admission_report,
    nc_output.spine_report
  )
  return nc_output
```

Guarantee:
1. cortex never dispatches execution directly.
2. spine executes admitted actions only.

## 2) Deterministic AttemptId Derivation

```text
fn derive_attempt_id(input):
  canonical = canonical_json({
    cycle_id,
    commitment_id,
    goal_id,
    planner_slot,
    affordance_key,
    capability_handle,
    normalized_payload,
    requested_resources,
    cost_attribution_id
  })
  return "att:" + hex(sha256(canonical))[0..24]
```

## 3) Cortex Step (Commitment + Scheduling + Attempt Plan)

```text
fn cortex.step(cmd):
  cycle_id += 1
  apply_goal_and_commitment_transitions(cmd)
  scheduling = recompute_scheduling_contexts(cycle_id)
  attempts = planner.plan_attempts(state, scheduling, cmd)
  assign planner_slot in stable sorted order
  attempts = attempts.map(derive_attempt_id + attach cost_attribution_id)
  reject duplicate attempt_id
  return attempts sorted by attempt_id
```

## 4) Non-Cortex Admission Pipeline

```text
fn non_cortex.process_attempts(cycle_id, attempts, spine):
  sorted = sort_by_attempt_id(attempts)
  admissions = []
  report_items = []

  for attempt in sorted:
    profile = resolve_affordance(attempt.affordance_key)
    if missing(profile):
      denied_hard("unknown_affordance")
      continue

    hard = hard_policy.check_hard(state, attempt, profile)
    if hard is Deny(code):
      denied_hard(code)
      continue

    base_cost = economic_policy.estimate_cost(state, attempt, profile)
    snap = cost_admission_policy.affordability(state, base_cost, attempt)
    reserve = cost_admission_policy.reserve_amount_micro(snap)

    if can_reserve(snap, reserve):
      reserve_entry_id = ledger.reserve(cycle_id, reserve, attempt.cost_attribution_id, ttl_cycles, ref)
      action = materialize_admitted(cycle_id, attempt, reserve_entry_id, degraded=false)
      admitted(action, snap, reserve_entry_id, reserve)
      continue

    candidate = deterministic_degrade_search(attempt, profile)
    if candidate exists:
      reserve_entry_id = ledger.reserve(...)
      action = materialize_admitted(... degraded=true ...)
      admitted(action, candidate.snap, reserve_entry_id, candidate.reserve)
    else:
      denied_economic("insufficient_survival_budget", snap)

  batch = AdmittedActionBatch(admissions)
  spine_report = spine.execute_admitted(batch)
  reconcile(spine_report)
  ingest_external_debits()
  return { outcomes, admission_report, spine_report, external_ledger_entry_ids }
```

## 5) Deterministic Degradation Search

```text
fn deterministic_degrade_search(attempt, profile):
  cands = degradation_policy.candidates(...)
  ranked = sort(cands, by configured tuple)
  capped = ranked.take(max_variants).respect_depth(max_depth)
  for cand in capped:
    if hard passes and affordability passes:
      return cand
  return none
```

Rank tuple:
1. if prefer-less-loss:
- `(capability_loss_score, estimated_survival_micro, profile_id)`
2. else:
- `(estimated_survival_micro, capability_loss_score, profile_id)`

## 6) Deterministic ActionId Derivation

```text
fn derive_action_id(cycle_id, source_attempt_id, reserve_entry_id):
  canonical = canonical_json({cycle_id, source_attempt_id, reserve_entry_id})
  return "act:" + hex(sha256(canonical))[0..24]
```

## 7) AdmissionReport Assembly

```text
fn denied_hard(code):
  outcome.disposition = DeniedHard{code}
  report.result = DeniedHard{code}

fn denied_economic(code, snap):
  outcome.disposition = DeniedEconomic{code}
  report.result = DeniedEconomic{code, affordability: snap}

fn admitted(action, snap, reserve_entry_id, reserve_amount, degraded):
  outcome.disposition = Admitted{degraded}
  outcome.admitted_action_id = action.action_id
  report.result = Admitted{
    degraded,
    reserve_entry_id,
    reserve_amount_micro: reserve_amount,
    degradation_profile_id: action.degradation_profile_id,
    affordability: snap
  }
```

## 8) Purity Guard (Admission)

```text
allowed inputs:
  NonCortexState
  attempt.affordance_key/capability_handle/requested_resources/normalized_payload
  deterministic registries/policies + version tuple

forbidden:
  wall-clock
  randomness
  unordered map iteration
  semantic narrative branching
```

## 9) Ordered Spine Event Reconciliation

```text
fn reconcile(spine_report):
  events = sort_by_seq_no(spine_report.events)
  for e in events:
    match e:
      ActionApplied{reserve_entry_id, actual_cost_micro, ...} =>
        adjust_vs_reserved(reserve_entry_id, actual_cost_micro)
        ledger.settle_reservation(reserve_entry_id, ref)
      ActionRejected{reserve_entry_id, ...} =>
        credit_reserved(reserve_entry_id)
        ledger.refund_reservation(reserve_entry_id, ref)
      _ => continue

  expire_open_reservations_if(cycle_id >= expires_at_cycle)
```

## 10) External Debit Ingestion

```text
fn ingest_external_debits():
  obs = debit_source.drain_observations(cursor)
  for o in obs:
    if seen_external_ref(o.reference_id): continue
    if !matches_attribution_chain(o.cost_attribution_id, o.action_id, o.cycle_id): continue
    ledger.apply_entry(debit from o)
    mark_seen(o.reference_id)
```

## 11) Settlement Idempotency

```text
settle/refund/expire(reserve_entry_id, reference_id):
  if reservation.terminal is Some and reservation.terminal_reference_id == Some(reference_id):
    return Ok(())         // idempotent replay
  if reservation.terminal is Some and reservation.terminal_reference_id != Some(reference_id):
    return Err(conflict)  // illegal second terminal transition
  apply terminal transition
```

Status: `READY_FOR_L3_REVIEW`
