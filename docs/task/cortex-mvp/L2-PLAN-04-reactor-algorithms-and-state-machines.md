# L2-04 - Reactor Algorithms And State Machines
- Task Name: `cortex-mvp`
- Stage: `L2` detailed file
- Date: `2026-02-11`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Always-On Reactor Loop
Canonical progression algorithm:

```text
start Reactor task
while inbox is open:
  receive ReactionInput (await)
  run react_once(input)
  send ReactionResult to outbox (await)
end when inbox closed
```

Properties:
1. no request/response entrypoint controls progression.
2. each inbox event advances exactly one reaction cycle.
3. backpressure is mechanical via bounded channel blocking/failure.

## 2) Per-Cycle State Machine
Cycle state machine:
1. `ReceivedInput`
2. `PrimaryIrReady`
3. `DraftsReady`
4. `Clamped`
5. `RepairedOnce` (optional)
6. `Completed`
7. `CompletedNoop`

Terminal states:
1. `Completed` (attempts may be empty or non-empty).
2. `CompletedNoop` (explicit fallback outcome).

## 3) `react_once` Algorithm
Pseudo-code:

```text
fn react_once(input):
  validate_input_bounds(input) or return noop
  init cycle_budget_guard from input.limits
  start cycle_deadline timer

  primary_ir = call_primary_once(input)
  if primary_ir fails or timeout: return noop

  drafts = call_extractor_once(primary_ir, input.capability_catalog, input.sense_window)
  if extractor fails or timeout: return noop

  first_clamp = clamp(drafts, input)
  if first_clamp.attempts not empty:
    return build_result(first_clamp)

  if repair_not_allowed_by_limits_or_budget:
    return noop

  repaired_drafts = call_filler_once(drafts, first_clamp.violations, input.capability_catalog)
  if filler fails or timeout:
    return noop

  second_clamp = clamp(repaired_drafts, input)
  if second_clamp.attempts empty:
    return noop

  return build_result(second_clamp)
```

Bound guarantees:
1. primary calls: exactly 1.
2. subcalls: extractor + optional filler only (`<= max_sub_calls`).
3. repair attempts: at most 1 (filler invocation after first clamp failure).

## 4) Deterministic Clamp Algorithm
Input:
1. draft attempts
2. reaction input
3. catalog/schemas/limits

Clamp steps:
1. canonicalize and pre-sort drafts by:
- `affordance_key`,
- `capability_handle`,
- canonical payload json,
- `intent_span`.
2. reject drafts with missing/empty `intent_span`.
3. reject drafts where `based_on` is empty or references unknown `sense_id`.
4. reject unknown `affordance_key` not in catalog.
5. reject unsupported `capability_handle` for chosen affordance.
6. enforce payload size:
- payload bytes <= min(`limits.max_payload_bytes`, affordance cap).
- oversize payload is dropped, not truncated.
7. validate payload against affordance schema.
8. clamp resources to allowed maxima and non-negative values.
9. derive deterministic `cost_attribution_id` and `attempt_id`.
10. stable-sort attempts by `attempt_id`.
11. enforce `max_attempts` by taking prefix after sort.

Output:
1. valid `IntentAttempt[]`.
2. `based_on` union list for `ReactionResult`.
3. `attention_tags` deterministic merge/dedupe/sort.
4. violation list for optional one-time repair.

## 5) Intent Arbitration Algorithm
Arbitration consumes distributed intent context plus current senses.

Deterministic process:
1. build candidate intent rows from:
- constitutional intents,
- environmental intent signals,
- emergent candidates.
2. compute deterministic rank key:
- source priority (`constitutional` > `environmental` > `emergent`),
- explicit urgency field if present,
- stable lexical tiebreak (`intent_key`/`candidate_key`).
3. produce arbitration summary inserted into primary prompt context.

Note:
- arbitration result is cycle-local; Cortex persists nothing durably.

## 6) Ingress Assembly Algorithm (Outside Cortex)
`CortexIngressAssembler` (runtime boundary) maintains latest non-cortex event material and emits bounded `ReactionInput`.

State held by assembler:
1. latest env snapshots
2. recent admission feedback window
3. latest capability catalog
4. latest reaction limits
5. latest intent context inputs

Trigger policy:
1. on `sense` event arrival, assemble and emit one `ReactionInput`.
2. on non-sense updates, update assembler state without forcing cycle unless configured.

No semantic decisions:
1. assembler performs structural aggregation only.
2. all intent reasoning remains in Cortex.

## 7) Backpressure Mechanics
1. inbox/outbox channels are bounded.
2. upstream uses `send().await` to naturally backpressure, or `try_send()` with deterministic rejection handling.
3. if upstream drops event due full queue, it must emit explicit dropped-count telemetry outside business flow.
4. Cortex never bypasses channel mechanics.

## 8) Timeout And Budget Policy
1. per-cycle wall-clock deadline from `max_cycle_time_ms`.
2. each model call receives bounded token and timeout limits.
3. once budget guard is exceeded, cycle short-circuits to noop.
4. no unbounded retries.

## 9) Noop Fallback Semantics
Noop means:
1. `ReactionResult.attempts = []`.
2. `ReactionResult.based_on` still reflects grounded senses when available.
3. `ReactionResult.attention_tags` may still be present if derived before failure.

Noop is emitted on:
1. invalid input bounds,
2. primary failure/timeout,
3. extractor failure/timeout,
4. clamp empty with no repair budget,
5. repair failure/timeout,
6. repaired clamp still empty.

## 10) Feedback Correlation Algorithm
Cross-layer correlation rule:
1. Cortex emits `attempt_id` in every `IntentAttempt`.
2. Admission/continuity feedback signals must include same `attempt_id`.
3. ingress assembler includes these correlated feedback signals in future `ReactionInput`.

This keeps "attempt is relative to world" contract explicit and replayable.

## 11) L2-04 Exit Criteria
This file is complete when:
1. run loop and cycle state machine are explicit,
2. clamp + repair policies are deterministic and bounded,
3. progression/backpressure responsibilities are mechanically assigned.
