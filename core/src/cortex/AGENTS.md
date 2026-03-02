# AGENTS.md for core/src/cortex

Cortex consumes `senses + physical_state + cognition_state` and emits:
1. `emitted_acts` (materialized acts with per-act wait metadata),
2. `new_cognition_state`,
3. control directives (for example sleep gate).

## Invariants
- Cortex can be called with empty senses on tick-driven cycles.
- Cortex is the cognition engine; Primary is the cognition core inside Cortex.
- Cortex does not durably persist cognition state internally.
- Input IR root is `<input-ir>` and Output IR root is `<output-ir>`.
- Primary accepts only assembled Input IR payload (no direct side channels).
- Primary runs a bounded cognitive micro-loop (`max_internal_steps`).
- Somatic act emission is tool-call-native; prompt text is not used to dispatch acts.
- Dynamic act tools are generated per turn with transport-safe aliases; aliases are mapped to fq act ids (`endpoint_id/neural_signal_descriptor_id`) in runtime code.
- Each dynamic act tool accepts:
  - `payload`
  - `wait_for_sense` (seconds, `0` means no wait)
- `wait_for_sense` is bounded by `max_waiting_seconds`.
- Internal static tools are:
  - `expand-senses`
  - `overwrite-sense-deferral-rule`
  - `reset-sense-deferral-rules`
  - `sleep`
  - `patch-goal-forest`
- `expand-senses` consumes `senses_to_expand[].sense_id` where `sense_id` is the rendered sense reference id:
  - `{monotonic_internal_sense_id}. {fq-sense-id}`
- Sense lines delivered to Primary are deterministic text:
  - `- [monotonic internal sense id]. [fq-sense-id]: [key=value,...]; [payload-truncated-if-needed]`
- Runtime wait implementation uses afferent deferral-rule control (overwrite/reset path), not a Stem scheduler hook.
- `patch-goal-forest` tool arguments are a direct JSON string of natural-language patch instructions; Primary does not author patch ops.
- `goal_forest_helper` runs a one-shot sub-agent to convert `current-goal-forest + patch-instructions` into JSON patch ops.
- `plant` adds a root node (`numbering=null`), while `sprout` adds a non-root node under a parent selector.
- Goal instincts are consolidated into Primary system prompt; there is no persisted root partition.
- Primary output text no longer carries `<somatic-acts>` / `<is-wait-for-sense>` contracts.
- Primary failure/timeout is fail-closed noop; helper failures degrade with deterministic fallback.
