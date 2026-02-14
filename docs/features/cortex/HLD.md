# Cortex HLD

## Module Boundary

Inputs:
- `Sense[]` (drained from queue for current cycle)
- `PhysicalState`
- `CognitionState`

Outputs:
- `CortexOutput`
  - `acts: Act[]` (or empty noop)
  - `new_cognition_state`

## Key Components

- `CortexPipeline`: single-cycle cognition orchestration.
- Cognition ports:
  - `PrimaryReasonerPort`
  - `AttemptExtractorPort`
  - `PayloadFillerPort`
- `DeterministicAttemptClamp`: final schema/catalog/capability authority.

## Invariants

- Cortex state is externalized; no durable internal persistence.
- `Act` remains non-binding and world-relative.
- deterministic clamp guards all emitted acts.
- business output remains clean (telemetry out-of-band).
