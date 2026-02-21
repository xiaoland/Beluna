# Cortex Module

Cortex is the stateless deliberative cognition module.

Code:
- `core/src/cortex/*`

Key properties:
- pure runtime boundary: `cortex(senses, physical_state, cognition_state) -> (acts, new_cognition_state)`
- no internal durable persistence of cognition state
- input/output IR pipeline (`<input-ir>` -> `<output-ir>`)
- input helpers (`sense_helper`, `act_descriptor_helper`) and output helpers (`acts_helper`, `goal_stack_helper`) are concurrent
- helpers are cognition organs: LLM handles semantic conversion/extraction, while deterministic assembly (catalog XML, IDs) is in Rust
- input helper payloads remove transport noise (for example `sense_id`) and use semantic `sense`/`act` naming for primary cognition
- primary failure is fail-closed noop; helper failures degrade via fallback/empty output
- emitted `Act[]` is deterministic and non-binding
