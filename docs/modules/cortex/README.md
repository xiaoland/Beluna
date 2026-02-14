# Cortex Module

Cortex is the stateless deliberative cognition module.

Code:
- `core/src/cortex/*`

Key properties:
- pure runtime boundary: `cortex(senses, physical_state, cognition_state) -> (acts, new_cognition_state)`
- no internal durable persistence of cognition state
- primary prose IR + sub-compile pipeline
- deterministic clamp as final authority
- emitted `Act[]` is deterministic and non-binding
