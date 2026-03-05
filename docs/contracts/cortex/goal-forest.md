# Goal Forest Contracts

This document defines the runtime contract of Goal Forest in Cortex.

## Scope

- Data model: `GoalNode`, `GoalForest`
- Internal tool: `patch-goal-forest`
- Continuity guardrails for persisted cognition state
- Deterministic ASCII rendering used by `<goal-forest>` input section

Primary implementation references:

- `/Users/lanzhijiang/Development/Beluna/core/src/cortex/helpers/goal_forest_helper/model.rs`
- `/Users/lanzhijiang/Development/Beluna/core/src/cortex/helpers/goal_forest_helper/mod.rs`
- `/Users/lanzhijiang/Development/Beluna/core/src/cortex/runtime/primary.rs`
- `/Users/lanzhijiang/Development/Beluna/core/src/continuity/state.rs`

## Data Model Contract

`GoalNode` fields:

- `id: string` (required, non-empty, globally unique across whole forest)
- `summary: string` (required, non-empty after trim)
- `status: string` (required, non-empty after trim)
- `weight: number` (required, finite and within `[0,1]`)
- `children: GoalNode[]` (required, use empty array when leaf)

`GoalForest` fields:

- `nodes: GoalNode[]` (forest roots)

## Internal Tool Contract

Tool name:

- `patch-goal-forest`

Arguments:

- object with:
  - `patch_instructions: string` (required)
  - `reset_context: boolean` (optional)

Execution behavior:

1. Runtime passes both current ASCII tree and current JSON tree to a goal-forest sub-agent.
2. Sub-agent returns strict JSON schema output: complete replacement `GoalNode[]`.
3. Runtime replaces the in-memory cycle-local forest with the returned list.
4. Runtime persists updated cognition via Continuity.

Return payload (`data`) includes:

- `previous_node_count`
- `replaced_node_count`
- `current_goal_forest` (ASCII)
- plus runtime fields from primary tool handler (for example persistence revision)

## ASCII Rendering Contract (`<goal-forest>`)

Rules:

- Empty forest fallback text:
  - `There's no trees in the goal forest currently.`
  - `Try to plan some trees, and patch them with complete GoalNode[] replacements.`
- Root line prefix: `+-- `
- Child line prefix per depth: `"    " * depth + "|-- "`
- Child numbering is derived by traversal path (1-based) and rendered for non-root nodes.
- Line format:
  - Root: `{prefix}[{status}] (w={weight:.2}) id={id} :: {summary}`
  - Non-root: `{prefix}{numbering} [{status}] (w={weight:.2}) id={id} :: {summary}`
- Order is deterministic by list order in `nodes` / `children`.

## Continuity Persistence Guardrails

Validation covers recursively:

- `id` uniqueness across entire forest
- non-empty `status` and `summary`
- valid `weight` range (`[0,1]`, finite)
- recursive validation of every child node

Persistence fails if any invariant is violated.

## Manual Checklist

1. Replace with one-root tree
- Send `patch-goal-forest` with instruction that asks for one root goal.
- Expect: `replaced_node_count = 1` and one rendered root line.

2. Replace with nested tree
- Ask for root plus two child goals.
- Expect: non-root lines render numbering `1`, `2`.

3. Replace and preserve existing
- Ask for “keep existing goals and add one child under X”.
- Expect: previous nodes remain and one additional node appears.

4. Invalid candidate rejection
- Force candidate with duplicate ids or invalid weight > 1.
- Expect: persistence guardrail failure from Continuity.

5. Empty replacement
- Ask to clear goal forest.
- Expect: fallback empty text in next `<goal-forest>` section.
