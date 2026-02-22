# Cortex LLD

## Domain Types

- `GoalTree`
  - `root_partition: string[]` (immutable, compile-time constant mirror)
  - `user_partition: GoalNode[]` (mutable flat forest)
- `GoalNode`
  - `numbering: string` (hierarchy path, for example `1`, `1.2`, `2.1.3`)
  - `node_id: string`
  - `summary: string`
  - `weight: float` (normalized `[0,1]`)
- `L1Memory`
  - `string[]`
- `CognitionState`
  - `revision: u64`
  - `goal_tree: GoalTree`
  - `l1_memory: L1Memory`

Patch ops:
- `GoalTreePatchOp`: `sprout | prune | tilt`

## Tick Algorithm

1. Build semantic helper inputs from `senses`, act descriptors, user goal tree.
2. Build `<input-ir>` with strict first-level XML sections.
3. Run primary to get `<output-ir>`.
4. Parse sections:
  - `<acts>`
  - `<goal-tree-patch>`
  - `<new-focal-awareness>`
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
- `l1_memory_flush_helper`: `string[]`

No envelope wrapper objects are allowed around helper outputs.

## Deterministic Patch Rules

- Goal-tree ops are applied to user partition only.
- Goal-tree ops are numbering-based only: `sprout(numbering, node_id, summary, weight)`, `prune(numbering)`, `tilt(numbering, weight)`.
- `sprout` fails closed if numbering is invalid or already exists.
- `prune` removes target numbering and descendants by prefix.
- `sprout` and `tilt` normalize incoming weights by dynamic Min-Max mapping from current forest weights.
- If current forest range is degenerate (`max == min`) or empty, normalization falls back to `0.5`.
- L1 flush applies full replacement semantics and truncates by `max_l1_memory_entries`.
- `revision` increments only when state changed.
