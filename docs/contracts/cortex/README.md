# Cortex Contracts

Boundary:
- input: `ReactionInput`
- output: `ReactionResult` with `IntentAttempt[]`

Must hold:
- reactor progression is inbox-event driven
- deterministic attempt derivation
- `IntentAttempt` is non-binding and world-relative
- non-noop attempts include `attempt_id` and `based_on: [sense_id...]`
- feedback path preserves `attempt_id` correlation
- one-primary/N-subcall/one-repair/noop bounded cycle policy
