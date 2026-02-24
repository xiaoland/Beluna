# Cortex LLD

## Domain Types

- `GoalTree`
  - `root_partition: string[]` (immutable, compile-time constant mirror)
  - `user_partition: GoalNode[]` (mutable flat forest)
- `GoalNode`
  - `numbering: string` (hierarchy path, for example `1`, `1.2`, `2.1.3`)
  - `weight: float` (normalized `[0,1]`)
  - `summary: string`
  - `content: string`
  - `status: string`
- `L1Memory`
  - `string[]`
- `CognitionState`
  - `revision: u64`
  - `goal_tree: GoalTree`
  - `l1_memory: L1Memory`

Patch ops:
- `GoalTreePatchOp`: `sprout | prune | tilt`

## Tick Algorithm

1. Run input helper instances (`sense`, `proprioception`, `act-descriptor`, `goal-tree`, `l1-memory`).
   - `sense_helper`: payload passthrough for small payloads, Postman Envelope for large payloads.
   - each `<somatic-sense>` is labeled with tick-local monotonic `sense-instance-id`.
   - `proprioception_input_helper`: deterministic map rendering into natural-language lines.
2. `goal_tree_input_helper` takes full `GoalTree` and consolidates both root-partition conversion (`<instincts>`) and user-partition conversion (`<willpower-matrix>`).
3. `l1_memory_input_helper` emits `<focal-awareness>` from current l1-memory; empty l1-memory uses deterministic markdown bullet one-shot.
4. Build `<input-ir>` with strict first-level XML sections.
5. Run Primary Cognitive Micro-loop (multi-turn chat).
6. During micro-loop, Primary may issue internal tool calls:
   - `expand-sense-raw(sense_ids)`
   - `expand-sense-with-sub-agent(tasks)`
   Tool transport is AI Gateway tool call; runtime appends assistant `tool_calls` (RPC request frame), then appends matched `tool` role replies (RPC response frame), and feeds both into next primary turn.
7. Stop micro-loop when final text `<output-ir>` is produced or `max_internal_steps` is reached.
8. Parse optional sections:
   - `<somatic-acts>` (missing => no acts)
   - `<willpower-matrix-patch>` (missing => empty goal-tree patch ops)
   - `<new-focal-awareness>` (missing => keep current l1-memory)
   - `<is-wait-for-sense>` (missing => false)
9. Run three output helper instances in parallel with contract `cognition-friendly input -> structured output`.
10. `acts_helper` returns final `Act[]` (including deterministic id materialization); Cortex applies patch arrays in deterministic order.
11. Return `CortexOutput { acts, new_cognition_state, wait_for_sense }`.

## IR Rules

- XML enforces only first-level boundaries.
- Section bodies are semi-structured markdown/plain text.
- Avoid high-entropy plumbing noise in IR.
- Keep semantic-first representation and implicit relational structure.
- `<somatic-senses>` and `<somatic-act-descriptor-catalog>` carry fully-qualified ids.
- `<somatic-senses>` additionally carries `sense-instance-id` (tick-local monotonic int), independent from external transport UUIDs.
- `<proprioception>` is a dedicated first-level section and does not use `sense-instance-id`.
- `sense` and `proprioception` are semantically distinct by contract.

## Helper JSON Contracts

- `sense_helper` large-payload envelope: `{ brief, original_size_in_bytes, confidence_score, omitted_features }`
- `sense_helper` small-payload path: raw payload JSON passthrough
- `acts_helper`: JSON array with fields `endpoint_id`, `fq_act_id`, `payload`, then helper-local deterministic materialization to `Act[]`
- `goal_tree_patch_helper`: `GoalTreePatchOp[]`
- `l1_memory_flush_helper`: `string[]`

No envelope wrapper objects are allowed around helper outputs.

## Deterministic Patch Rules

- Goal-tree ops are applied to user partition only.
- Goal-tree ops are numbering-based only: `sprout(numbering, weight, summary, content, status)`, `prune(numbering)`, `tilt(numbering, weight)`.
- `sprout` fails closed if numbering is invalid or already exists.
- `prune` removes target numbering and descendants by prefix.
- `sprout` and `tilt` normalize incoming weights by dynamic Min-Max mapping from current forest weights.
- If current forest range is degenerate (`max == min`) or empty, normalization falls back to `0.5`.
- L1 flush applies full replacement semantics and truncates by `max_l1_memory_entries`.
- `revision` increments only when state changed.
