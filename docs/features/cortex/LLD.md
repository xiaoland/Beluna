# Cortex LLD

## Pipeline

Single cycle execution:

1. `input_helpers(senses, physical_state, cognition_state) -> <input-ir>`
2. `primary(<input-ir>) -> <output-ir>`
3. `output_helpers(<output-ir>) -> acts + goal_stack_patch`
4. `goal_stack_patch` is applied to current cognition state to produce `new_cognition_state`

## IR Contract

- Input IR must contain root `<input-ir>` and first-level sections:
  - `<senses>`
  - `<act-descriptor-catalog>`
  - `<goal-stack>`
  - `<context>`
- Output IR must contain root `<output-ir>` and first-level sections:
  - `<acts>`
  - `<goal-stack-patch>`
- Section body is semi-structured XML/Markdown.

## Helper Semantics

- Input helpers run concurrently:
  - `sense_helper`
  - `act_descriptor_helper`
- Output helpers run concurrently:
  - `acts_helper`
  - `goal_stack_helper`
- Every helper is one LLM call.
- Helper model route is configurable per-helper with default fallback.

## Cache

- `act_descriptor_helper` cache key is MD5 of canonical act-descriptor input.
- Cache store is in-memory, process-scoped.

## Failure Policy

- Primary failure or timeout: fail-closed noop (`acts=[]`) and cognition state remains unchanged.
- Input helper failure: fallback to raw section content.
- Output helper failure: fallback to empty output for that helper (`acts=[]` or empty patch).

## Output Constraints

- `acts_helper` and `goal_stack_helper` use AI Gateway JSON Schema strict mode.
- `act_id` is generated in code as UUIDv7.
