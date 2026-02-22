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

Primary Output IR is internal protocol only and is not `CortexOutput`.

## Key Components

- `runtime.rs`: cycle orchestration and helper fan-out/fan-in.
- `ir.rs`: `<input-ir>` and `<output-ir>` envelope assembly/parsing.
- `helpers_input.rs`: semantic projection for input helper calls.
- `helpers_output.rs`: output helper parsing plus patch application.
- `prompts.rs`: primary prompt and helper prompts in one module.
- `cognition.rs`: `GoalTree`, `GoalNode`, `L1Memory`, patch op contracts.

## Organ Topology

Input helper stage (parallel):
- `sense_helper`
- `act_descriptor_helper` with MD5 process cache
- `goal_tree_helper` with MD5 process cache on user partition

Primary stage:
- `primary(<input-ir>) -> <output-ir>`

Output helper stage (parallel):
- `acts_helper` -> `ActDraft[]`
- `goal_tree_patch_helper` -> `GoalTreePatchOp[]`
- `l1_memory_patch_helper` -> `L1MemoryPatchOp[]`

Patch application is done inside Cortex to produce full `new_cognition_state`.

## Invariants

- Root partition is immutable and never modified by Cortex patching.
- User partition is a flat forest mutated by numbering-based `sprout/prune/tilt`; L1 memory is mutable through allowed op sets.
- Primary behavior invariants are expressed by prompt contract, not hard-coded policy branches.
- Helper protocol remains semantic-first and plumbing-free.
