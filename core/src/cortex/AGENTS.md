# AGENTS.md for core/src/cortex

Cortex is a stateless cognition boundary that consumes `senses + physical_state + cognition_state` and emits `Act[] + new_cognition_state + wait_for_sense`.

## Invariants
- Cortex can be called with empty senses on tick-driven cycles.
- Cortex is the cognition engine; Primary is the cognition core inside Cortex.
- Sense helper bypasses LLM calls when domain senses are empty and uses deterministic fallback section output.
- Cortex does not durably persist cognition/goal state internally.
- Input IR root is `<input-ir>` and Output IR root is `<output-ir>`.
- Primary is the cognition core: it reasons from assembled section context (`<senses>`, `<act-descriptor-catalog>`, `<instincts>`, `<willpower-matrix>`, `<focal-awareness>`) and decides intent; it is not a generic IR transformation engine.
- Primary is not an LLM wrapper concept.
- Helper modules are cognition organs that reduce Primary cognition-load.
- `<input-ir>` / `<output-ir>` are deterministic Rust-owned internal envelopes and are not exposed to Primary LLM prompts/contracts.
- Primary contract accepts only assembled Input IR payload (no direct `senses`, `physical_state`, or `cognition_state` side channels).
- Primary output includes `<is-wait-for-sense>` (true/false) to explicitly control whether Stem should wait for a new sense before next Active tick; default behavior is false.
- Input helpers (`sense_helper`, `act_descriptor_helper`, `goal_tree_helper`) run concurrently to assemble Input IR sections.
- Output helpers (`acts_helper`, `goal_tree_patch_helper`, `l1_memory_flush_helper`) run concurrently from Output IR sections.
- Each helper is implemented as a dedicated submodule under `core/src/cortex/helpers/`.
- Runtime (`runtime.rs`) orchestrates only boundary state and IR flow; helper conversion implementation is owned by helper modules.
- `act_descriptor_helper` cache is in-memory and process-scoped, keyed by act-descriptor MD5 input hash.
- `goal_tree_helper` cache is in-memory and process-scoped, keyed by user-partition MD5 input hash.
- `act_descriptor_helper` input uses raw descriptor fields and composes `fq_act_id` internally; XML wrapping is deterministic Rust code.
- `sense_helper` input uses raw sense fields and composes `fq_sense_id` internally; XML wrapping is deterministic Rust code.
- `goal_tree_helper` receives the complete goal-tree (`root_partition + user_partition`) at the helper boundary.
- Root partition conversion to `<instincts>` is consolidated inside `goal_tree_input_helper` (deterministic Rust), while helper cognition conversion/caching remains driven by `user_partition`.
- Goal-tree helper bypasses LLM calls when user partition is empty and returns deterministic one-shot pursuits in `<willpower-matrix>`.
- Empty `l1_memory` is replaced by deterministic one-shot bullet statements in `<focal-awareness>`.
- Output helpers consume section-local context only and do not require full Output IR text.
- Input helper contract shape: structured input -> cognition-friendly output.
- Output helper contract shape: cognition-friendly input -> structured output.
- Input helper payloads passed to LLM are semantic projections: transport ids like `sense_instance_id` are filtered out.
- Primary failure/timeout is fail-closed noop; helper failures degrade by fallback sections/empty outputs.
- `acts_helper` owns act structuring/materialization and generates `act_instance_id` in code (UUIDv7), not by LLM.
- Primary IR carries fully-qualified sense/act ids only and excludes instance ids.
- `CortexOutput` is runtime boundary output (`acts + new_cognition_state`), distinct from Primary Output IR.
