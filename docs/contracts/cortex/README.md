# Cortex Contracts

Boundary:
- input: `CortexCommand`
- output: `CortexCycleOutput` with `IntentAttempt[]`

Must hold:
- deterministic attempt derivation
- goal/commitment separation
- dynamic scheduling per cycle
- failed commitments include failure code
