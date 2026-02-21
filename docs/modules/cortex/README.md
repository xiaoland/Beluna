# Cortex Module

Cortex is the stateless deliberative cognition module.

Code:
- `core/src/cortex/*`

Key properties:
- pure runtime boundary: `cortex(senses, physical_state, cognition_state) -> (acts, new_cognition_state)`
- no internal durable persistence of cognition state
- Primary is the cognition core: it deliberates over sensed context and current goals to decide acts and goal-stack updates
- `<input-ir>` and `<output-ir>` are boundary contracts, not a generic transformation objective
- deterministic Rust code assembles/parses IR contracts; cognition organs perform semantic reasoning/extraction
- input helpers (`sense_helper`, `act_descriptor_helper`) and output helpers (`acts_helper`, `goal_stack_helper`) are concurrent
- helpers are cognition organs: LLM handles semantic conversion/extraction, while deterministic assembly (catalog XML, IDs) is in Rust
- input helper payloads remove transport noise (for example `sense_id`) and use semantic `sense`/`act` naming for primary cognition
- primary failure is fail-closed noop; helper failures degrade via fallback/empty output
- emitted `Act[]` is deterministic and non-binding
- observability events include `input_ir_sense`, `input_ir_act_descriptor_catalog`, `output_ir_acts`, and `final_returned_acts`
- Cortex metrics include `beluna_cortex_cycle_id` and `beluna_cortex_input_ir_act_descriptor_catalog_count`
