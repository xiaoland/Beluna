# Goal Forest Contracts

This document defines the runtime contract of Goal Forest in Cortex, including patch tool behavior and manual test cases.

## Scope

- Data model: `GoalNode`, `GoalForest`, `GoalForestPatchOp`
- Internal tool: `patch-goal-forest`
- Deterministic mutation rules in Cortex patch applier
- Continuity guardrails for persisted cognition state
- Deterministic ASCII rendering used by `<goal-forest>` input section

Primary implementation references:

- `/Users/lanzhijiang/Development/Beluna/core/src/cortex/cognition.rs`
- `/Users/lanzhijiang/Development/Beluna/core/src/cortex/cognition_patch.rs`
- `/Users/lanzhijiang/Development/Beluna/core/src/cortex/runtime.rs`
- `/Users/lanzhijiang/Development/Beluna/core/src/cortex/helpers/goal_forest_input_helper.rs`
- `/Users/lanzhijiang/Development/Beluna/core/src/continuity/state.rs`

## Data Model Contract

`GoalNode` fields:

- `id: string` (required, non-empty, globally unique)
- `summary: string` (required, non-empty after trim)
- `status: string` (required, non-empty after trim)
- `weight: number` (required, finite and within `[0,1]`)
- `parent_id: string | null`
- `numbering: string | null`

Topology rules:

- Root node: `parent_id = null`, `numbering = null`
- Non-root node: `parent_id != null`, `numbering != null`
- Non-root `numbering` must be direct-child numbering of parent:
  - Parent numbering `null` => child numbering is single segment (for example `1`)
  - Parent numbering `1.2` => child numbering must be `1.2.N` (exactly one segment deeper)
- Numbering uniqueness is scoped to siblings (same `parent_id`), not global across forest
- Parent chain must be acyclic

Numbering format when present:

- Dot-separated positive integer segments
- No `0` segment
- No leading zeros (for example `01`)

## Internal Tool Contract

Tool name:

- `patch-goal-forest`

Arguments:

- Top-level JSON array only (not wrapped object)
- Must contain at least one operation
- Schema source of truth is hard-coded in:
  - `/Users/lanzhijiang/Development/Beluna/core/src/cortex/runtime.rs`
- Mirror copy:
  - `/Users/lanzhijiang/Development/Beluna/core/src/constant/patch-goal-forest.json`

Tool execution behavior:

- If arguments JSON fails to parse against the op enum, tool returns error (`ok: false`)
- If JSON parses but an op is semantically invalid, that op is ignored (no state change), and tool still returns success
- Tool return `data` is the full updated goal forest in ASCII-art format

## Patch Operation Semantics

### `plant`

Intent:

- Add a new root node (new tree root)

Required:

- `op = "plant"`
- `id`
- `summary`

Optional:

- `status` (default `"open"`)
- `weight` (default `0.0`)

Validation:

- Reject duplicate `id`
- Reject empty `id`/`summary`/`status`
- Reject invalid `weight`
- Root insertion forces `parent_id = null`, `numbering = null`

### `sprout`

Intent:

- Add a non-root node under an existing parent

Required:

- `op = "sprout"`
- `id`
- `summary`
- Parent selector: at least one of `parent_id` or `parent_numbering`

Optional:

- `numbering` (if omitted, auto-assign next direct child numbering)
- `status` (default `"open"`)
- `weight` (default `0.0`)

Selector behavior:

- If both `parent_id` and `parent_numbering` are provided, both must resolve to the same parent
- `parent_numbering` selector must resolve uniquely; ambiguous numbering across trees is treated as invalid selector

Validation:

- Reject duplicate `id`
- Reject empty `id`/`summary`/`status`
- Reject invalid `weight`
- Parent must exist
- Final numbering must be a direct child numbering of parent
- Sibling numbering under the same `parent_id` must be unique

Auto-numbering:

- If parent numbering is `null`, children are `1`, `2`, `3`, ...
- If parent numbering is `1.3`, children are `1.3.1`, `1.3.2`, ...

### `trim`

Intent:

- Update selected node fields

Required:

- `op = "trim"`
- Selector: at least one of `id` or `numbering`
- At least one update field: `weight` or `status`

Validation:

- Selector must resolve to exactly one node
- If both selectors are present, they must resolve to the same node
- `weight` must be finite and in `[0,1]`
- `status` must be non-empty after trim

Behavior:

- Applies only provided fields
- No-op if provided values equal existing values

### `prune`

Intent:

- Remove selected node and all descendants

Required:

- `op = "prune"`
- Selector: at least one of `id` or `numbering`

Validation:

- Selector must resolve to exactly one node
- If both selectors are present, they must resolve to the same node

Behavior:

- Descendants are determined by parent linkage (`parent_id` closure), not numbering prefix

## ASCII Rendering Contract (`<goal-forest>`)

Rendering source:

- `/Users/lanzhijiang/Development/Beluna/core/src/cortex/helpers/goal_forest_input_helper.rs`

Rules:

- Empty forest fallback text:
  - `There's no trees in the goal forest currently.`
  - `Try to plan some trees, and then plant, sprout, prune, trim them.`
- Root line prefix: `+-- `
- Child line prefix per depth: `"    " * depth + "|-- "`
- Display numbering:
  - Root node: omitted from rendered line
  - Non-root node: actual numbering string
- Line format:
  - Root: `{prefix}[{status}] (w={weight:.2}) id={id} :: {summary}`
  - Non-root: `{prefix}{numbering} [{status}] (w={weight:.2}) id={id} :: {summary}`

Ordering:

- Render from roots (`parent_id = null`) and recurse depth-first
- Siblings sorted by numbering then id
- Unreachable/orphan nodes (if any) are still emitted as depth `0` after rooted traversal

## Continuity Persistence Guardrails

Guardrails are enforced when persisting cognition state:

- `/Users/lanzhijiang/Development/Beluna/core/src/continuity/state.rs`

Validation covers:

- `id` uniqueness
- Non-empty `status` and `summary`
- Valid `weight` range
- Root/non-root topology constraints
- Parent existence
- Direct-child numbering consistency
- Sibling numbering uniqueness per parent
- Parent cycle detection

Persistence fails if any invariant is violated.

## Legacy Load Compatibility

Deserializer migration is applied for legacy flat numbering trees:

- Legacy root nodes (`"1"`, `"2"`, ...) become roots with `numbering = null`
- Legacy descendants map to `parent_id` via numbering chain and get relative numbering (for example legacy `1.2.3` becomes child with numbering `2.3` under legacy `1`)

If migration cannot be resolved deterministically, original nodes are kept as-is.

## Manual Test Checklist

Use `patch-goal-forest` tool with JSON array arguments.

1. Plant root success
- Input op: `plant` with unique `id` and `summary`
- Expect: one node inserted with `parent_id = null`, `numbering = null`

2. Plant duplicate id
- Plant same `id` twice
- Expect: second op no effect, tool still returns success

3. Sprout under root by `parent_id`
- Parent root has `numbering = null`
- Sprout without `numbering`
- Expect: child numbering auto-assigned to `"1"` then `"2"` for subsequent siblings

4. Sprout under non-root by `parent_numbering`
- Sprout under parent numbering `"1"`
- Expect: child numbering auto-assigned as `"1.1"`, `"1.2"`, ...

5. Sprout with explicit invalid numbering
- Example: provide `"2.1"` under parent numbering `"1"`
- Expect: op no effect

6. Sprout selector mismatch
- Provide `parent_id` and `parent_numbering` pointing to different nodes
- Expect: op no effect

7. Trim success
- Trim existing node by `id`, set `status` and `weight`
- Expect: values updated

8. Trim ambiguous numbering
- Create two roots, each with child numbering `"1"`
- Trim by `numbering: "1"` only
- Expect: selector ambiguity, op no effect

9. Prune subtree
- Prune a non-root parent with children
- Expect: target and all descendants removed

10. Continuity rejection: invalid topology
- Manually persist state containing root with non-null numbering
- Expect: continuity invariant violation

11. Continuity rejection: cycle
- Manually persist state where parent chain loops
- Expect: continuity invariant violation

12. Render check
- Verify `<goal-forest>` root lines do not render literal `null` numbering
- Verify prefixes and ordering are deterministic
