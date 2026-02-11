# Cortex HLD

## Module Boundary

Inputs:
- `CortexCommand`
- prior `CortexState`
- optional `AdmissionReport` feedback

Outputs:
- updated `CortexState`
- `CortexCycleOutput` with deterministic `IntentAttempt[]`

## Key Components

- `CommitmentManager`: goal/commitment transitions and invariant checks.
- `DeterministicPlanner`: dynamic scheduling and attempt derivation.
- `CortexFacade`: cycle orchestrator.

## Invariants

- Goal identity and commitment lifecycle are separate.
- `Failed` commitment requires `failure_code`.
- Supersession is a relationship (`superseded_by`), not a commitment status.
