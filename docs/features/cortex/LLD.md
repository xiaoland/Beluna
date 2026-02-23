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

1. Run input helper instances (`sense`, `act-descriptor`, `goal-tree`) with contract `structured input -> cognition-friendly output`; helpers compose fully-qualified ids internally when needed.
2. `goal_tree_input_helper` takes full `GoalTree` and consolidates both root-partition conversion (`<instincts>`) and user-partition conversion (`<willpower-matrix>`).
3. Build `<input-ir>` with strict first-level XML sections.
4. Run primary to get `<output-ir>`.
5. Parse sections:
  - `<acts>`
  - `<goal-tree-patch>`
  - `<new-focal-awareness>`
6. Run three output helper instances in parallel with contract `cognition-friendly input -> structured output`.
7. `acts_helper` returns final `Act[]` (including deterministic id materialization); Cortex applies patch arrays in deterministic order.
8. Return `CortexOutput { acts, new_cognition_state, wait_for_sense }`.

## IR Rules

- XML enforces only first-level boundaries.
- Section bodies are semi-structured markdown/plain text.
- Avoid high-entropy plumbing noise in IR.
- Keep semantic-first representation and implicit relational structure.
- `<senses>` and `<act-descriptor-catalog>` carry fully-qualified ids only; they do not carry instance ids.

## Helper JSON Contracts

- `acts_helper`: JSON array with fields `endpoint_id`, `fq_act_id`, `payload`, then helper-local deterministic materialization to `Act[]`
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
