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

- `runtime.rs`: cycle orchestration and IR assembly/parsing; helper-level timeout/fallback is consolidated inside each helper module.
- `ir.rs`: `<input-ir>` and `<output-ir>` envelope assembly/parsing.
- `helpers/`: one helper per submodule (`sense_input_helper`, `act_descriptor_input_helper`, `goal_tree_input_helper`, `l1_memory_input_helper`, `acts_output_helper`, `goal_tree_patch_output_helper`, `l1_memory_flush_output_helper`).
- `cognition_patch.rs`: deterministic cognition patch application.
- `prompts.rs`: primary prompt and helper prompts in one module.
- `cognition.rs`: `GoalTree`, `GoalNode`, `L1Memory`, patch op contracts.

## Organ Topology

Input helper stage (parallel):
- `sense_helper`
- `act_descriptor_helper` with MD5 process cache
- `goal_tree_helper` receives full goal-tree and keeps MD5 process cache keyed on user partition
- `l1_memory_input_helper` emits focal-awareness section deterministically

Primary stage:
- `primary-micro-loop(<input-ir>, internal-tool-calls) -> <output-ir>`
- Primary is the cognition engine core, not an LLM wrapper concept.
- Internal tool calls are Internal Cognitive Actions (`expand-sense-raw`, `expand-sense-with-sub-agent`), not Somatic Acts.
- Micro-loop is bounded by `max_internal_steps`.

Output helper stage (parallel):
- `acts_helper` -> `Act[]`
- `goal_tree_patch_helper` -> `GoalTreePatchOp[]`
- `l1_memory_flush_helper` -> `string[]`

Patch application is done inside Cortex to produce full `new_cognition_state`.

## Invariants

- Root partition is immutable and never modified by Cortex patching.
- User partition is a flat forest mutated by numbering-based `sprout/prune/tilt`; node shape is `numbering, weight, summary, content, status`; L1 memory is mutable through allowed op sets.
- Output helpers consume section-local context only and do not require full Output IR text.
- Sense helper conversion contract: structured input -> Postman Envelope JSON for large payloads; passthrough JSON for small payloads.
- Sense entries expose `sense-instance-id` (tick-local monotonic int) and `fq-somatic-sense-id` attributes.
- Input helper conversion contract (non-sense): structured input -> cognition-friendly output.
- Output helper conversion contract: cognition-friendly input -> structured output.
- Primary behavior invariants are expressed by prompt contract, not hard-coded policy branches.
- Helper protocol remains semantic-first and plumbing-free.
- Input/Output IR uses fully-qualified somatic sense/act ids only and excludes instance ids.
