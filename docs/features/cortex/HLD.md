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
- Input helper stage:
  - `sense_helper`
  - `act_descriptor_helper` (process-level in-memory cache)
- Primary stage:
  - `primary(<input-ir>) -> <output-ir>`
- Output helper stage:
  - `acts_helper` (JSON Schema strict mode)
  - `goal_stack_helper` (JSON Schema strict mode)

## Invariants

- Cortex state is externalized; no durable internal persistence.
- `Act` remains non-binding and world-relative.
- primary failure/timeout is fail-closed noop.
- helper failure degrades to fallback section/empty helper result.
- business output remains clean (telemetry out-of-band).
