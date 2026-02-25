# Cortex LLD

## Domain Types

- `GoalForest`
  - `nodes: GoalNode[]` (mutable flat forest)
- `GoalNode`
  - `numbering: string` (hierarchy path, for example `1`, `1.2`, `2.1.3`)
  - `status: string`
  - `weight: float` (`[0,1]`)
  - `id: string`
  - `summary: string`
- `L1Memory`
  - `string[]`
- `CognitionState`
  - `revision: u64`
  - `goal_forest: GoalForest`
  - `l1_memory: L1Memory`

Patch ops:
- `GoalForestPatchOp`: `sprout | plant | trim | prune`

## Tick Algorithm

1. Run input helper instances (`sense`, `proprioception`, `act-descriptor`, `goal-forest`, `l1-memory`).
2. `goal_forest_input_helper` renders GoalForest as deterministic ASCII-art (`+--`, `|--`).
3. Build `<input-ir>` with first-level sections.
4. Run Primary Cognitive Micro-loop.
5. During micro-loop, Primary may issue internal tools:
   - `expand-sense-raw(sense_ids)`
   - `expand-sense-with-sub-agent(tasks)`
   - `patch-goal-forest(ops[])`
6. `patch-goal-forest` applies deterministic ops to cycle-local GoalForest and returns updated ASCII-art.
7. Stop micro-loop when final text `<output-ir>` is produced or `max_internal_steps` is reached.
8. Parse optional output sections:
   - `<somatic-acts>` (missing => no acts)
   - `<new-focal-awareness>` (missing => keep current l1-memory)
   - `<is-wait-for-sense>` (missing => false)
9. Run output helpers in parallel:
   - `acts_helper` -> `Act[]`
   - `l1_memory_flush_helper` -> `L1Memory`
10. Compose `new_cognition_state` from cycle-local GoalForest + flushed L1Memory.
11. Increment `revision` iff cognition changed.

## IR Rules

- XML enforces only first-level boundaries.
- Input sections:
  - `<somatic-senses>`
  - `<proprioception>`
  - `<somatic-act-descriptor-catalog>`
  - `<goal-forest>`
  - `<focal-awareness>`
- Output sections:
  - `<somatic-acts>`
  - `<new-focal-awareness>`
  - `<is-wait-for-sense>`
- There is no output goal-forest patch section.

## Helper JSON Contracts

- `sense_helper` large-payload envelope: `{ brief, original_size_in_bytes, confidence_score, omitted_features }`
- `sense_helper` small-payload path: raw payload JSON passthrough
- `acts_helper`: JSON array with fields `endpoint_id`, `fq_act_id`, `payload`
- `l1_memory_flush_helper`: `string[]`

## Deterministic GoalForest Patch Rules

- Selector rule: for `trim`/`prune`, at least one selector (`numbering` or `id`) is required.
- Parent selector rule: for `sprout`, at least one parent selector (`parent_numbering` or `parent_id`) is required.
- Selector consistency: if both `numbering` and `id` are provided, both must resolve to the same node.
- `plant`:
  - adds a new root goal node with `numbering=null`
  - root nodes have no parent (`parent_id=null`)
  - defaults: `status="open"`, `weight=0.0`
- `sprout`:
  - resolves parent via (`parent_numbering` | `parent_id`)
  - if `numbering` is omitted, auto-assign next direct child numbering under parent
  - if `numbering` is provided, it must be a direct child numbering of resolved parent
  - non-root nodes must always have `parent_id` and non-null `numbering`
  - numbering uniqueness is enforced among siblings (same parent), not globally across roots
- `trim`: updates `weight` and/or `status` on selected node.
- `prune`: removes selected node and descendants by parent linkage.
- Weight policy: no normalization; values must be finite and in `[0,1]`.
- L1 flush applies full replacement semantics and truncates by `max_l1_memory_entries`.
