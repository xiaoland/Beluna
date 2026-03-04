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
- `wait_for_sense > 0` is valid only when the target act descriptor declares non-empty `emitted_sense_ids`.
- Wait-for-sense gating is runtime tick-skip based:
  - no afferent deferral rule mutation is performed by wait-for-sense logic
  - completion requires a buffered sense that matches both:
    - `sense.act_instance_id == dispatched act_instance_id`
    - `fq-sense-id` in descriptor-declared `emitted_sense_ids`
- Internal static tools are:
  - `expand-senses`
  - `add-sense-deferral-rule`
  - `remove-sense-deferral-rule`
  - `sleep`
  - `patch-goal-forest`
- `expand-senses` consumes `senses_to_expand[].sense_id` where `sense_id` is the rendered sense reference id:
  - `{monotonic_internal_sense_id}. {fq-sense-id}`
- Sense internal ids are process-lifetime monotonic (not per cycle).
- Sense lines delivered to Primary are deterministic text:
  - `- [monotonic internal sense id]. [fq-sense-id]: [key=value,...]; [payload-truncated-if-needed]`
- `patch-goal-forest` tool arguments are a direct JSON string of natural-language patch instructions; Primary does not author patch ops.
- `goal_forest_helper` runs a one-shot sub-agent to convert `current-goal-forest + patch-instructions` into JSON patch ops.
- Goal forest model + patch logic live under `helpers/goal_forest_helper`.
- `plant` adds a root node (`numbering=null`), while `sprout` adds a non-root node under a parent selector.
- Goal instincts are consolidated into Primary system prompt; there is no persisted root partition.
- Primary output text no longer carries `<somatic-acts>` / `<is-wait-for-sense>` contracts.
- Primary failure/timeout is fail-closed noop; helper failures degrade with deterministic fallback.
