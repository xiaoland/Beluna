# Cortex Module

Cortex is the stateless deliberative cognition module.

Code:

- `core/src/cortex/*`

Key properties:

- pure runtime boundary: `cortex(senses, physical_state, cognition_state) -> (acts, new_cognition_state, wait_for_sense)`
- no internal durable persistence of cognition state
- Cortex is the cognition engine; Primary is the cognition core inside Cortex.
- cognition state model: `goal-tree` + `l1-memory`
- root partition is compile-time immutable string array
- user partition is a mutable flat weighted goal forest (`GoalNode[]`) with hierarchy encoded by `numbering` (for example `1`, `1.2`, `2.1.3`); each node carries `numbering`, `weight`, `summary`, `content`, `status`
- `<input-ir>` and `<output-ir>` are boundary contracts, not a generic transformation objective
- primary user prompt sections are assembled as `<somatic-senses>`, `<somatic-act-descriptor-catalog>`, `<instincts>`, `<willpower-matrix>`, `<focal-awareness>`
- deterministic Rust code assembles/parses IR contracts; cognition organs perform semantic reasoning/extraction
- input helpers (`sense_helper`, `act_descriptor_helper`, `goal_tree_helper`, `l1_memory_input_helper`) and output helpers (`acts_helper`, `goal_tree_patch_helper`, `l1_memory_flush_helper`) are concurrent
- each helper is implemented as its own submodule under `core/src/cortex/helpers/`
- helpers are cognition organs that reduce Primary cognition-load; LLM handles semantic conversion/extraction, while deterministic assembly (catalog XML, IDs) is in Rust
- runtime (`core/src/cortex/runtime.rs`) only orchestrates state/IR stages; helper conversion and helper fallback are owned by helper modules
- Primary is a Cognitive Micro-loop and uses AI Gateway tool calls for Internal Cognitive Actions.
- Internal Cognitive Action tools:
  - `expand-sense-raw(sense_ids)`
  - `expand-sense-with-sub-agent(tasks)`
  - both consume tick-local monotonic `sense-instance-id`.
- `sense_helper` takes raw sense inputs, assigns tick-local monotonic `sense-instance-id`, composes `fq_sense_id`, and emits `<somatic-sense sense-instance-id="..." fq-somatic-sense-id="...">...`
- `sense_helper` emits Postman Envelope JSON (`brief`, `original_size_in_bytes`, `confidence_score`, `omitted_features`) for large payloads, and passthrough JSON for small payloads (`sense_passthrough_max_bytes`)
- `act_descriptor_helper` takes raw descriptors, internally composes `fq_act_id`, and emits `<somatic-act-descriptor somatic-act-id="...">...`
- goal-tree helper boundary receives full goal-tree (`root_partition + user_partition`); root conversion is consolidated inside helper module
- goal-tree helper cache remains keyed by user partition forest
- when `user_partition` is empty, goal-tree section uses deterministic one-shot pursuits plus patch shot guidance
- when `l1_memory` is empty, focal-awareness section uses deterministic one-shot bullet statements
- input helper direction is:
  - `sense_helper`: `structured input -> postman-envelope|passthrough output`
  - other input helpers: `structured input -> cognition-friendly output`
- output helper direction is `cognition-friendly input -> structured output`
- `acts_helper` owns act structuring/materialization and returns final `Act[]`
- primary failure is fail-closed noop; helper failures degrade via fallback/empty output
- emitted `Act[]` is deterministic and non-binding
- Primary Input/Output IR carries fully-qualified neural signal ids; senses additionally carry monotonic `sense-instance-id`
- observability events include `input_ir_sense`, `input_ir_act_descriptor_catalog`, `output_ir_goal_tree_patch`, `output_ir_l1_memory_flush`, and `final_returned_acts`
- Cortex metrics include `beluna_cortex_cycle_id` and `beluna_cortex_input_ir_act_descriptor_catalog_count`
