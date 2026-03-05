# Cortex LLD

## Domain Types

- `GoalForest`
  - `nodes: GoalNode[]` (forest roots)
- `GoalNode`
  - `status: string`
  - `weight: float` (`[0,1]`)
  - `id: string`
  - `summary: string`
  - `children: GoalNode[]`
- `L1Memory`
  - `string[]`
- `CognitionState`
  - `revision: u64`
  - `goal_forest: GoalForest`
  - `l1_memory: L1Memory`

## Tick Algorithm

1. Run input helper instances (`sense`, `proprioception`, `act-descriptor`, `goal-forest`, `l1-memory`).
2. `goal_forest_input_helper` renders GoalForest as deterministic ASCII-art (`+--`, `|--`) with derived numbering from traversal path.
3. Build `<input-ir>` with first-level sections.
4. Run one Primary thread turn.
5. During a turn, Primary may issue internal tools:
   - `expand-senses(tasks[])` where each task uses `sense_id` and optional `use_subagent_and_instruction_is`
   - `patch-goal-forest(patch_instructions)`
6. `patch-goal-forest` invokes one sub-agent with `current-goal-forest + patch-instructions`, receives full `GoalNode[]`, and replaces cycle-local GoalForest in one shot.
7. If tick `N` turn returns tool-calls, runtime buffers tool-result messages and injects them into tick `N+1` turn, along with tick `N+1` input payload, until final text `<output-ir>` is produced.
8. Parse final `<output-ir>` only when no tool-calls remain.
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
- Output is parsed only as `<output-ir>` envelope text for contract validation.

## Helper JSON Contracts

- `sense_helper`: deterministic text line per sense: `- <monotonic-id>. endpoint_id=<endpoint_id>, sense_id=<sense_id>, weight=<weight>[, truncated_ratio=<0..1 if truncated>]; payload="<payload-truncated-if-needed>"`
- `goal_forest_helper`: strict JSON schema output `GoalNode[]` full replacement
- `acts_helper`: JSON array with fields `endpoint_id`, `fq_act_id`, `payload`
- `l1_memory_flush_helper`: `string[]`

## Goal Forest Replacement Rules

- Full replacement semantics: helper output is the entire next forest state, not patch ops.
- All node ids must be globally unique across the full forest.
- `status` and `summary` must be non-empty.
- `weight` must be finite and in `[0,1]`.
- `children` must always be an array.
- L1 flush applies full replacement semantics and truncates by `max_l1_memory_entries`.
