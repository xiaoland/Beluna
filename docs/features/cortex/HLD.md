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

- `runtime.rs`: cycle orchestration, IR assembly/parsing, timeout/fallback policy.
- `ir.rs`: `<input-ir>` and `<output-ir>` envelope assembly/parsing.
- `helpers/`: one helper per submodule (`sense_input_helper`, `act_descriptor_input_helper`, `goal_tree_input_helper`, `acts_output_helper`, `goal_tree_patch_output_helper`, `l1_memory_flush_output_helper`).
- `cognition_patch.rs`: deterministic cognition patch application.
- `prompts.rs`: primary prompt and helper prompts in one module.
- `cognition.rs`: `GoalTree`, `GoalNode`, `L1Memory`, patch op contracts.

## Organ Topology

Input helper stage (parallel):
- `sense_helper`
- `act_descriptor_helper` with MD5 process cache
- `goal_tree_helper` receives full goal-tree and keeps MD5 process cache keyed on user partition

Primary stage:
- `primary(<input-ir>) -> <output-ir>`
- Primary is the cognition engine core, not an LLM wrapper concept.

Output helper stage (parallel):
- `acts_helper` -> `Act[]`
- `goal_tree_patch_helper` -> `GoalTreePatchOp[]`
- `l1_memory_flush_helper` -> `string[]`

Patch application is done inside Cortex to produce full `new_cognition_state`.

## Invariants

- Root partition is immutable and never modified by Cortex patching.
- User partition is a flat forest mutated by numbering-based `sprout/prune/tilt`; L1 memory is mutable through allowed op sets.
- Output helpers consume section-local context only and do not require full Output IR text.
- Input helper conversion contract: structured input -> cognition-friendly output.
- Output helper conversion contract: cognition-friendly input -> structured output.
- Primary behavior invariants are expressed by prompt contract, not hard-coded policy branches.
- Helper protocol remains semantic-first and plumbing-free.
- Input/Output IR uses fully-qualified sense/act ids only and excludes instance ids.
