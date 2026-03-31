# L3-03 - Core Pseudocode
- Task Name: `cortex-mvp`
- Stage: `L3` detail: core execution logic
- Date: `2026-02-11`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Top-Level Reactor Run
```text
async fn run_reactor(mut inbox, mut outbox, deps):
  while let Some(input) = inbox.recv().await:
    result = react_once(input, deps).await
    if outbox.send(result).await fails:
      break
```

Guarantees:
1. one reaction per inbox item.
2. always-on until inbox/outbox closure.

## 2) Single Cycle Algorithm
```text
async fn react_once(input, deps):
  if !validate_input_bounds(input):
    return noop_result(input.reaction_id, input.sense_window)

  budget = new_budget_guard(input.limits)
  deadline = now + input.limits.max_cycle_time_ms

  arbitration = arbitrate_intents(input.context, input.sense_window)

  ir = primary_once(arbitration, input, deps.primary, budget, deadline).await
  if ir failed:
    return noop_result(...)

  drafts = extractor_once(ir, input, deps.extractor, budget, deadline).await
  if drafts failed:
    return noop_result(...)

  clamped_1 = clamp(drafts, input, deps.clamp)
  if clamped_1.attempts not empty:
    return build_result(input.reaction_id, clamped_1)

  if !budget.can_repair_once():
    return noop_result(...)

  repaired = filler_once(
    clamped_1.original_drafts,
    clamped_1.violations,
    input,
    deps.filler,
    budget,
    deadline
  ).await
  if repaired failed:
    return noop_result(...)

  clamped_2 = clamp(repaired, input, deps.clamp)
  if clamped_2.attempts empty:
    return noop_result(...)

  return build_result(input.reaction_id, clamped_2)
```

## 3) Intent Arbitration Pseudocode
```text
fn arbitrate_intents(ctx, senses):
  rows = []
  rows += map constitutional intents -> rank_source=0
  rows += map environmental signals -> rank_source=1
  rows += map emergent candidates -> rank_source=2

  rows.sort_by(
    rank_source,
    urgency_if_present,
    stable_key
  )
  return summary(rows, senses)
```

## 4) Deterministic Clamp Pseudocode
```text
fn clamp(drafts, input, clamp_deps):
  violations = []
  attempts = []

  sorted = stable_sort(drafts by affordance_key, capability_handle, canonical_payload, intent_span)

  for draft in sorted:
    if intent_span missing -> violations += MissingIntentSpan; continue
    if draft.based_on empty -> violations += MissingBasedOn; continue
    if any based_on not in input.sense_window -> violations += UnknownSenseId; continue

    profile = catalog.resolve(draft.affordance_key)
    if none -> violations += UnknownAffordance; continue

    if draft.capability_handle not allowed by profile ->
      violations += UnsupportedCapabilityHandle; continue

    payload = canonicalize(draft.payload_draft)
    payload_bytes = bytes(payload)
    if payload_bytes > min(input.limits.max_payload_bytes, profile.max_payload_bytes):
      violations += PayloadTooLarge; continue

    if !json_schema_validate(payload, profile.payload_schema):
      violations += PayloadSchemaViolation; continue

    resources = clamp_resources(draft.requested_resources, input.limits)

    planner_slot = next_slot()
    cost_id = derive_cost_attribution_id(input.reaction_id, draft, planner_slot)
    attempt_id = derive_attempt_id(input.reaction_id, draft, payload, resources, cost_id)

    attempts += IntentAttempt{
      attempt_id,
      based_on = stable_dedupe_sort(draft.based_on),
      affordance_key = draft.affordance_key,
      capability_handle = draft.capability_handle,
      normalized_payload = payload,
      requested_resources = resources,
      cost_attribution_id = cost_id,
      ...
    }

  attempts.sort_by(attempt_id)
  attempts = attempts.take(input.limits.max_attempts)

  return { attempts, violations, original_drafts=drafts }
```

## 5) One-Repair Guard
```text
struct BudgetGuard {
  primary_calls: u8
  sub_calls: u8
  repair_calls: u8
  ...
}

primary_once():
  require primary_calls == 0
  primary_calls += 1

extractor_once():
  require sub_calls < max_sub_calls
  sub_calls += 1

filler_once():
  require repair_calls == 0
  require sub_calls < max_sub_calls
  repair_calls += 1
  sub_calls += 1
```

## 6) Noop Result
```text
fn noop_result(reaction_id, sense_window):
  return ReactionResult {
    reaction_id,
    based_on = sense_ids_from_window(sense_window),
    attention_tags = [],
    attempts = []
  }
```

## 7) Feedback Correlation Rule
```text
for each attempt emitted:
  require attempt.attempt_id present

for each admission feedback signal ingested:
  require signal.attempt_id present

assembler stores feedback window by attempt_id
next reaction input includes correlated non-semantic codes
```

## 8) Adapter-to-AIGateway Call Skeleton
```text
async fn primary_adapter(req):
  ai_req = BelunaInferenceRequest {
    output_mode = Text,
    tools = [],
    limits.max_output_tokens = req.limits.max_primary_output_tokens,
    stream = false,
    ...
  }
  resp = gateway.infer_once(ai_req).await?
  return ProseIr { text: resp.output_text }
```

Extractor/filler adapters:
1. use required tool schema outputs.
2. parse tool args JSON into typed drafts.
3. on parse failure, return stage error and let cycle noop.

Status: `READY_FOR_L3_REVIEW`
