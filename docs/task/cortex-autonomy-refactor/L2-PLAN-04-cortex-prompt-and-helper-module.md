# L2-04 Cortex Prompt and Helper Module
- Task: `cortex-autonomy-refactor`
- Stage: `L2`

## 1) Unified Prompt Module
Create single module:
- `core/src/cortex/prompts.rs`

This module is the only owner of prompt text for:
1. Primary system+user prompt builders
2. Input helpers
3. Output helpers

No inline prompt literals remain in `runtime.rs`.

## 2) Primary Prompt Design Rule
Your provided primary system prompt is baseline; implementation will refine wording but keep the same invariant intent.

Important boundary:
1. invariants are expressed in system prompt instructions,
2. not hard-coded as behavior logic in Rust (except deterministic contract enforcement like XML section parsing).

Primary prompt output contract remains:
1. output only `<output-ir>`
2. required sections: `<acts>`, `<goal-tree-patch>`, `<l1-memory-patch>`

## 3) Input Helper Set
Input helpers run concurrently:
1. `sense_helper`
2. `act_descriptor_helper`
3. `goal_tree_helper`

Helper routing:
1. each helper route configurable,
2. `goal_tree_helper` gets cache behavior same as `act_descriptor_helper` (hash-keyed process cache).

Goal-tree helper scope:
1. receives only user partition (not root partition).
2. root partition strings are injected directly by deterministic runtime.

L1-memory handling:
1. no helper,
2. passthrough ordered `string[]` directly into `<l1-memory>`.

## 4) Output Helper Set
Output helpers run concurrently:
1. `acts_helper`
2. `goal_tree_patch_helper`
3. `l1_memory_patch_helper`

JSON schema outputs have no extra wrapper objects:
1. `acts_helper` returns `ActDraft[]`
2. `goal_tree_patch_helper` returns `GoalTreePatchOp[]`
3. `l1_memory_patch_helper` returns `L1MemoryPatchOp[]`

## 5) Cortex Internal Pipeline
1. assemble InputIR sections (`senses`, `act-descriptor-catalog`, `goal-tree`, `l1-memory`).
2. call primary with system prompt carrying invariants.
3. parse output-ir sections.
4. run 3 output helpers concurrently.
5. apply parsed patches inside Cortex to current cognition state.
6. return final `CortexOutput { acts, new_cognition_state }`.

## 6) Empty Sense Behavior
Cortex accepts empty `senses` in active ticks and may still produce proactive acts.

## 7) IR Section Policy
1. first-level XML sections only (strict parser).
2. no extra `<context>` section.
3. no “helper content/body notes” channel.

## 8) Telemetry Updates
Rename/add telemetry keys aligned with new contracts:
1. `input_ir_goal_tree`
2. `input_ir_l1_memory`
3. `output_ir_goal_tree_patch`
4. `output_ir_l1_memory_patch`

