# Cortex PRD

## Purpose

Cortex is Beluna's always-on deliberative reactor.

Cortex consumes bounded `ReactionInput` deltas, runs one bounded cognition cycle, and emits non-binding `IntentAttempt[]`.

## Requirements

- Cortex progression is inbox-event driven, not request/response.
- Cortex does not durably persist goals/commitments.
- Primary LLM outputs prose IR; sub-LLM stages compile IR to structured drafts.
- Deterministic clamp is final authority before attempts leave Cortex.
- `attempt_id` and `cost_attribution_id` are deterministic.
- Every non-noop attempt includes `based_on: [sense_id...]`.
- Cortex can intend anything; execution is constrained downstream.
- Per-cycle hard bounds:
  - exactly 1 primary call,
  - at most N subcalls,
  - at most 1 repair call,
  - strict max attempts/payload/time/token limits,
  - fallback to noop on irreparable failures.

## Out of Scope

- Direct execution access.
- Constraint narration or moral interpretation.
