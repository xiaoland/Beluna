# Data And State

## System-Wide State Shapes

1. Cognition state:
- persisted by Continuity.
- includes goal-forest and memory structures used by Cortex runtime.

2. Physical state snapshot:
- ledger snapshot.
- neural-signal descriptor catalog.
- proprioception map.

3. Signal identity:
- descriptor identity: `sense_id` / `act_id` (within endpoint scope).
- fully qualified identity: `endpoint_id/neural_signal_descriptor_id`.
- runtime event correlation: instance IDs (`sense_instance_id`, `act_instance_id`).

## Contract/Guardrail Placement

- Stable contracts live in Product/Unit TDD interface and operation docs.
- Enforcement guardrails live in code-level validators, schema generation, and tests.
