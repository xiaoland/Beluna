# Cortex Contracts

Boundary:
1. input: `ReactionInput`
2. output: `ReactionResult` with `IntentAttempt[]`

Must hold:
1. reactor progression is inbox-event driven
2. deterministic attempt derivation
3. `IntentAttempt` is non-binding and world-relative
4. non-noop attempts include `attempt_id` and `based_on: [sense_id...]`
5. feedback path preserves `attempt_id` correlation
6. one-primary/N-subcall/one-repair/noop bounded cycle policy
7. capability catalog consumed by Cortex is sourced from Spine capability snapshot bridge
