# AGENTS.md for core/src/cortex

Cortex consumes `senses + physical_state`.

Cortex output is control directives only (for example sleep gate).

Cognition state is read/persisted through Continuity directly from Cortex/Primary tool handlers.

## Invariants
- Cortex can be called with empty senses on tick-driven cycles.
- Cortex is the cognition engine; Primary is the cognition core inside Cortex.
- Input IR root is `<input-ir>` and Output IR root is `<output-ir>`.
- Primary accepts only assembled Input IR payload (no direct side channels).
- Cortex runtime is tick-driven only: one AI Gateway thread turn per admitted tick.
- If a tick turn returns tool calls, tool results are buffered and injected into the next admitted tick turn.
- A continuation tick turn carries both:
  - previous tick tool result messages
  - current tick input payload (including senses accumulated since the prior admitted tick)
- Somatic act emission is tool-call-native; prompt text is not used to dispatch acts.
- Dynamic act tools are generated per turn with transport-safe aliases; aliases are mapped to fq act ids (`endpoint_id/neural_signal_descriptor_id`) in runtime code.
- Dynamic act tool alias derivation is deterministic from fq act id:
  - `.` -> `-`
  - `/` -> `_`
- Each dynamic act tool accepts:
  - `payload`
  - `wait_for_sense` (ticks, `0` means no explicit sense-wait request)
- Dynamic act tool return includes `ActDispatchResult` from the serial efferent pipeline (Continuity -> Spine), bounded by timeout (`lost` on timeout).
- `wait_for_sense > 0` is valid only when the target act carries non-empty `might_emit_sense_ids`.
- Wait-for-sense gating is runtime tick-skip based:
  - no afferent deferral rule mutation is performed by wait-for-sense logic
  - completion requires a buffered sense that matches both:
    - `sense.act_instance_id == dispatched act_instance_id`
    - `fq-sense-id` in `act.might_emit_sense_ids`
- Internal static tools are:
  - `expand-senses`
  - `add-sense-deferral-rule`
  - `remove-sense-deferral-rule`
  - `sleep`
  - `patch-goal-forest`
- `expand-senses` consumes a direct JSON array of tasks (`tasks[]`) as tool arguments:
  - each task requires `sense_id` as rendered sense reference id (`{monotonic_internal_sense_id}`)
  - optional `use_subagent_and_instruction_is` switches that task to sub-agent expansion
- Sense internal ids are process-lifetime monotonic (not per cycle).
- Sense lines delivered to Primary are deterministic text:
  - `- [monotonic internal sense id]. endpoint_id=[endpoint_id], sense_id=[sense_id], weight=[weight][, truncated_ratio=[0..1 if truncated]]; payload="[payload-truncated-if-needed]"`
- `patch-goal-forest` tool arguments are a direct JSON string of natural-language patch instructions.
- `goal_forest_helper` runs a one-shot sub-agent to convert `current-goal-forest + patch-instructions` into a complete replacement `GoalNode[]`.
- Goal forest model + replacement logic live under `helpers/goal_forest_helper`.
- Goal node hierarchy is nested through `children` arrays (not `parent_id/numbering` selectors).
- Goal instincts are consolidated into Primary system prompt; there is no persisted root partition.
- Primary output text no longer carries `<somatic-acts>` / `<is-wait-for-sense>` contracts.
- Primary failure/timeout is fail-closed noop; helper failures degrade with deterministic fallback.
