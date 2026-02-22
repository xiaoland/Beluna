# Cortex Module

Cortex is the stateless deliberative cognition module.

Code:
- `core/src/cortex/*`

Key properties:
- pure runtime boundary: `cortex(senses, physical_state, cognition_state) -> (acts, new_cognition_state)`
- no internal durable persistence of cognition state
- cognition state model: `goal-tree` + `l1-memory`
- root partition is compile-time immutable string array
- user partition is a mutable flat weighted goal forest (`GoalNode[]`) with hierarchy encoded by `numbering` (for example `1`, `1.2`, `2.1.3`)
- `<input-ir>` and `<output-ir>` are boundary contracts, not a generic transformation objective
- deterministic Rust code assembles/parses IR contracts; cognition organs perform semantic reasoning/extraction
- input helpers (`sense_helper`, `act_descriptor_helper`, `goal_tree_helper`) and output helpers (`acts_helper`, `goal_tree_patch_helper`, `l1_memory_patch_helper`) are concurrent
- helpers are cognition organs: LLM handles semantic conversion/extraction, while deterministic assembly (catalog XML, IDs) is in Rust
- input helper payloads remove transport noise (for example `sense_id`) and use semantic `sense`/`act` naming for primary cognition
- goal-tree helper only receives user partition forest and is cache-enabled
- l1-memory is passed through as ordered `string[]`
- primary failure is fail-closed noop; helper failures degrade via fallback/empty output
- emitted `Act[]` is deterministic and non-binding
- observability events include `input_ir_sense`, `input_ir_act_descriptor_catalog`, `output_ir_acts`, and `final_returned_acts`
- Cortex metrics include `beluna_cortex_cycle_id` and `beluna_cortex_input_ir_act_descriptor_catalog_count`
