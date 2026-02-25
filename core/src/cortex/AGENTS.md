# AGENTS.md for core/src/cortex

Cortex is a stateless cognition boundary that consumes `senses + physical_state + cognition_state` and emits `Act[] + new_cognition_state + wait_for_sense`.

## Invariants
- Cortex can be called with empty senses on tick-driven cycles.
- Cortex is the cognition engine; Primary is the cognition core inside Cortex.
- Cortex does not durably persist cognition state internally.
- Input IR root is `<input-ir>` and Output IR root is `<output-ir>`.
- Primary accepts only assembled Input IR payload (no direct side channels).
- Primary runs a bounded cognitive micro-loop (`max_internal_steps`).
- Internal tools are Internal Cognitive Actions, not Somatic Act outputs.
- Internal tools are `expand-sense-raw`, `expand-sense-with-sub-agent`, `patch-goal-forest`.
- `patch-goal-forest` tool arguments are a top-level JSON array of ops.
- `plant` adds a root node (`numbering=null`), while `sprout` adds a non-root node under a parent selector.
- Goal instincts are consolidated into Primary system prompt; there is no persisted root partition.
- Input IR sections are `<somatic-senses>`, `<proprioception>`, `<somatic-act-descriptor-catalog>`, `<goal-forest>`, `<focal-awareness>`.
- `<goal-forest>` is ASCII-art forest text rendered with `+--` / `|--` by deterministic Rust code.
- Primary does not emit goal-forest patch sections in Output IR; goal updates happen through `patch-goal-forest` tool calls.
- Primary output sections are optional; missing sections degrade deterministically:
  - `<somatic-acts>` => no acts
  - `<new-focal-awareness>` => keep current l1-memory
  - `<is-wait-for-sense>` => false
- Input helpers (`sense_helper`, `proprioception_input_helper`, `act_descriptor_helper`, `goal_forest_input_helper`, `l1_memory_input_helper`) run concurrently.
- Output helpers (`acts_helper`, `l1_memory_flush_helper`) run concurrently.
- `goal_forest_input_helper` is deterministic (no LLM call).
- `act_descriptor_helper` cache is in-memory and process-scoped (MD5 input hash).
- Sense helper assigns tick-local monotonic integer `sense-instance-id`; internal sense expansion tools consume these IDs.
- `acts_helper` owns act structuring/materialization and generates `act_instance_id` in code (UUIDv7).
- Primary failure/timeout is fail-closed noop; helper failures degrade with deterministic fallback.
