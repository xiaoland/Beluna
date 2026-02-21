# AGENTS.md for core/src/cortex

Cortex is a stateless cognition boundary that consumes a drained sense batch plus physical/cognition snapshots and emits `Act[]` + next cognition state.

## Invariants
- Progression is input-event driven only.
- Cortex does not durably persist cognition/goal state internally.
- Input IR root is `<input-ir>` and Output IR root is `<output-ir>`.
- Input helpers (`sense_helper`, `act_descriptor_helper`) run concurrently to assemble Input IR sections.
- Output helpers (`acts_helper`, `goal_stack_helper`) run concurrently from Output IR sections.
- `act_descriptor_helper` cache is in-memory and process-scoped, keyed by act-descriptor MD5 input hash.
- `act_descriptor_helper` only converts one `<act-descriptor>` payload to markdown; catalog XML wrapping and `<input-ir>` assembly are deterministic Rust code.
- Input helper payloads passed to LLM are semantic projections: transport ids like `sense_id` are filtered out, and `sense`/`act` naming is used instead of `neural_signal_descriptor`.
- Primary failure/timeout is fail-closed noop; helper failures degrade by fallback sections/empty outputs.
- `act_id` is generated in code (UUIDv7), not by LLM.
