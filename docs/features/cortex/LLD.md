# Cortex LLD

## Domain Types

- `GoalTree`
  - `root_partition: string[]` (immutable, compile-time constant mirror)
  - `user_partition: GoalNode` (mutable tree)
- `GoalNode`
  - `node_id: string`
  - `summary: string`
  - `weight: i32`
  - `children: GoalNode[]`
- `L1Memory`
  - `entries: string[]`
- `CognitionState`
  - `revision: u64`
  - `goal_tree: GoalTree`
  - `l1_memory: L1Memory`

Patch ops:
- `GoalTreePatchOp`: `sprout | prune | tilt`
- `L1MemoryPatchOp`: `append | insert | remove`

## Tick Algorithm

1. Build semantic helper inputs from `senses`, act descriptors, user goal tree.
2. Build `<input-ir>` with strict first-level XML sections.
3. Run primary to get `<output-ir>`.
4. Parse sections:
  - `<acts>`
  - `<goal-tree-patch>`
  - `<l1-memory-patch>`
5. Run three output helpers in parallel with strict JSON Schema outputs.
6. Materialize `Act[]` and apply patch arrays in deterministic order.
7. Return `CortexOutput { acts, new_cognition_state }`.

## IR Rules

- XML enforces only first-level boundaries.
- Section bodies are semi-structured markdown/plain text.
- Avoid high-entropy plumbing noise in IR.
- Keep semantic-first representation and implicit relational structure.

## Helper JSON Contracts

- `acts_helper`: `ActDraft[]`
- `goal_tree_patch_helper`: `GoalTreePatchOp[]`
- `l1_memory_patch_helper`: `L1MemoryPatchOp[]`

No envelope wrapper objects are allowed around helper outputs.

## Deterministic Patch Rules

- Goal-tree ops are applied to user partition only.
- `sprout` fails closed if parent is missing or node_id already exists.
- `prune` cannot remove `user-root`.
- `tilt` clamps weight to configured bounds.
- L1 ops apply on ordered list index semantics.
- `revision` increments only when state changed.
