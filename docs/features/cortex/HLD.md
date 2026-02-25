# Cortex HLD

## Module Boundary

Inputs:
- `senses: Sense[]`
- `physical_state: PhysicalState`
- `cognition_state: CognitionState`

Outputs:
- `CortexOutput`
  - `acts: Act[]`
  - `new_cognition_state: CognitionState`
  - `wait_for_sense: bool`

Primary Output IR is internal protocol only and is not `CortexOutput`.

## Key Components

- `runtime.rs`: cycle orchestration, primary micro-loop, tool routing, and final cognition assembly.
- `ir.rs`: `<input-ir>` and `<output-ir>` envelope assembly/parsing.
- `helpers/`: one helper per submodule (`sense_input_helper`, `proprioception_input_helper`, `act_descriptor_input_helper`, `goal_forest_input_helper`, `l1_memory_input_helper`, `acts_output_helper`, `l1_memory_flush_output_helper`).
- `cognition_patch.rs`: deterministic cognition patch application.
- `prompts.rs`: primary prompt and helper prompts.
- `cognition.rs`: `GoalForest`, `GoalNode`, `L1Memory`, patch op contracts.

## Organ Topology

Input helper stage (parallel):
- `sense_helper`
- `proprioception_input_helper` (deterministic map-to-natural-language rendering)
- `act_descriptor_helper` with MD5 process cache
- `goal_forest_input_helper` (deterministic GoalForest -> ASCII-art `<goal-forest>`)
- `l1_memory_input_helper`

Primary stage:
- `primary-micro-loop(<input-ir>, internal-tool-calls) -> <output-ir>`
- Internal tools:
  - `expand-sense-raw`
  - `expand-sense-with-sub-agent`
  - `patch-goal-forest`
- `patch-goal-forest` updates cycle-local goal-forest state and returns updated ASCII-art.

Output helper stage (parallel):
- `acts_helper` -> `Act[]`
- `l1_memory_flush_helper` -> `string[]`

Final cognition state is composed inside Cortex from:
- cycle-local goal-forest state produced by `patch-goal-forest`
- l1-memory flush output

## Invariants

- Goal instincts are in primary system prompt (no persisted root partition).
- Goal-forest node shape is `numbering, status, weight, id, summary`.
- Goal weights must already be valid `[0,1]`; no normalization.
- Output IR has no goal-forest patch section.
- Output helpers consume section-local context only.
- Sense helper conversion contract: structured input -> Postman Envelope JSON for large payloads; passthrough JSON for small payloads.
- Proprioception helper conversion contract: `BTreeMap<String, String> -> deterministic natural-language section`.
- Sense entries expose `sense-instance-id` (tick-local monotonic int) and `fq-somatic-sense-id` attributes.
- Input IR contains `<proprioception>` section distinct from `<somatic-senses>`.
- Input/Output IR uses fully-qualified somatic sense/act ids only and excludes instance ids.
