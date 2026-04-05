# Route: Reality

## Trigger

Use when a bug, outage, anomaly, failing test, or runtime observation shows that reality diverges from expectation.

## Primary Owner

- Evidence first in `tasks/`
- Recurrence tripwires in the nearest local `AGENTS.md` when warranted
- Durable fixes in `20-product-tdd`, `30-unit-tdd`, `40-deployment`, or code/tests after the cause is justified

## Common Mode Overlays

- `Diagnose` first
- `Explore` or `Execute` only after evidence supports the next step

## Forbidden

- No evidence, no modification.
- Do not collapse multiple hypotheses into one guess.

## Read-Do Steps

1. Capture the observable symptom, timeline, and blast radius.
2. Collect logs, metrics, traces, failing tests, or other direct evidence.
3. Rank likely causes and make missing evidence explicit.
4. Apply the fix only after the next step is justified.
5. Add or refine the nearest local `AGENTS.md` tripwire if the same class of mistake is likely to recur.

## Exit Criteria

- Likely causes are ranked with evidence.
- The next action is justified rather than guessed.
- Recurrence guardrails are updated when needed.
