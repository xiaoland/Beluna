# AGENTS.md for core/src/cortex

Cortex is a stateless cognition boundary that consumes `senses + physical_state + cognition_state` and emits `Act[] + new_cognition_state`.

## Invariants
- Cortex can be called with empty senses on tick-driven cycles.
- Cortex does not durably persist cognition/goal state internally.
- Input IR root is `<input-ir>` and Output IR root is `<output-ir>`.
- Primary is the cognition core: it reasons from `<input-ir>` context and decides intent; it is not a generic IR transformation engine.
- `<input-ir>` / `<output-ir>` are protocol envelopes around cognition, while deterministic Rust code enforces shape and orchestration.
- Input helpers (`sense_helper`, `act_descriptor_helper`, `goal_tree_helper`) run concurrently to assemble Input IR sections.
- Output helpers (`acts_helper`, `goal_tree_patch_helper`, `l1_memory_patch_helper`) run concurrently from Output IR sections.
- `act_descriptor_helper` cache is in-memory and process-scoped, keyed by act-descriptor MD5 input hash.
- `goal_tree_helper` cache is in-memory and process-scoped, keyed by user-partition MD5 input hash.
- `act_descriptor_helper` only converts one `<act-descriptor>` payload to markdown; catalog XML wrapping and `<input-ir>` assembly are deterministic Rust code.
- `goal_tree_helper` only receives user partition; root partition remains deterministic Rust-owned immutable context.
- Input helper payloads passed to LLM are semantic projections: transport ids like `sense_id` are filtered out, and `sense`/`act` naming is used instead of `neural_signal_descriptor`.
- Primary failure/timeout is fail-closed noop; helper failures degrade by fallback sections/empty outputs.
- `act_id` is generated in code (UUIDv7), not by LLM.
- `CortexOutput` is runtime boundary output (`acts + new_cognition_state`), distinct from Primary Output IR.
