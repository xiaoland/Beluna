# AGENTS.md for core/src/cortex

Cortex is a stateless cognition boundary that consumes `senses + physical_state + cognition_state` and emits `Act[] + new_cognition_state + wait_for_sense`.

## Invariants
- Cortex can be called with empty senses on tick-driven cycles.
- Cortex is the cognition engine; Primary is the cognition core inside Cortex.
- Sense helper bypasses LLM calls when domain senses are empty and uses deterministic fallback section output.
- Cortex does not durably persist cognition/goal state internally.
- Input IR root is `<input-ir>` and Output IR root is `<output-ir>`.
- Primary is the cognition core: it reasons from assembled section context (`<somatic-senses>`, `<proprioception>`, `<somatic-act-descriptor-catalog>`, `<instincts>`, `<willpower-matrix>`, `<focal-awareness>`) and decides intent; it is not a generic IR transformation engine.
- Primary is not an LLM wrapper concept.
- Cognitive Sovereignty belongs to Primary: Primary runs a multi-turn Cognitive Micro-loop instead of one-shot inference.
- Internal cognitive tool calls are Internal Cognitive Actions, not Somatic Act outputs.
- Cognitive Micro-loop is bounded by `max_internal_steps` to prevent infinite loops.
- Helper modules are cognition organs that reduce Primary cognition-load.
- `<input-ir>` / `<output-ir>` are deterministic Rust-owned internal envelopes and are not exposed to Primary LLM prompts/contracts.
- Primary contract accepts only assembled Input IR payload (no direct `senses`, `physical_state`, or `cognition_state` side channels).
- Primary output sections are optional; missing sections degrade deterministically (`<somatic-acts>` => no acts, `<willpower-matrix-patch>` => no goal-tree ops, `<new-focal-awareness>` => keep current l1-memory). `<is-wait-for-sense>` defaults to false when absent.
- Input helpers (`sense_helper`, `proprioception_input_helper`, `act_descriptor_helper`, `goal_tree_helper`, `l1_memory_input_helper`) run concurrently to assemble Input IR sections.
- Output helpers (`acts_helper`, `goal_tree_patch_helper`, `l1_memory_flush_helper`) run concurrently from Output IR sections.
- Each helper is implemented as a dedicated submodule under `core/src/cortex/helpers/`.
- Runtime (`runtime.rs`) orchestrates only boundary state and IR flow; helper conversion implementation is owned by helper modules.
- `act_descriptor_helper` cache is in-memory and process-scoped, keyed by act-descriptor MD5 input hash.
- `goal_tree_helper` cache is in-memory and process-scoped, keyed by user-partition MD5 input hash.
- `act_descriptor_helper` input uses raw descriptor fields and composes `fq_act_id` internally; XML wrapping is deterministic Rust code.
- `sense_helper` assigns tick-local monotonic integer `sense-instance-id` and composes `fq_sense_id` internally; both are encoded as `<somatic-sense ...>` XML attributes for Primary.
- `expand-sense-raw` and `expand-sense-with-sub-agent` internal tools consume `sense-instance-id` values only.
- `goal_tree_helper` receives the complete goal-tree (`root_partition + user_partition`) at the helper boundary.
- Root partition conversion to `<instincts>` is consolidated inside `goal_tree_input_helper` (deterministic Rust), while helper cognition conversion/caching remains driven by `user_partition`.
- Goal-tree helper bypasses LLM calls when user partition is empty and returns deterministic one-shot pursuits in `<willpower-matrix>`.
- Goal-tree helper empty-user fallback includes a deterministic patch shot for initial sprout operations.
- Empty `l1_memory` is replaced by deterministic one-shot markdown bullet statements in `<focal-awareness>`.
- Output helpers consume section-local context only and do not require full Output IR text.
- Sense helper contract shape: structured input -> Postman Envelope (`brief`, `original_size_in_bytes`, `confidence_score`, `omitted_features`) when payload is large.
- Sense helper passthrough: when payload bytes <= `sense_passthrough_max_bytes`, payload is forwarded directly.
- Input helper contract shape (non-sense): structured input -> cognition-friendly output.
- Output helper contract shape: cognition-friendly input -> structured output.
- Input helper payloads passed to LLM are semantic projections: external transport ids (for example runtime UUID `SenseDatum.sense_instance_id`) are filtered out.
- Primary failure/timeout is fail-closed noop; helper failures degrade by fallback sections/empty outputs.
- `acts_helper` owns act structuring/materialization and generates `act_instance_id` in code (UUIDv7), not by LLM.
- Primary IR carries fully-qualified somatic sense/act ids only and excludes instance ids.
- `CortexOutput` is runtime boundary output (`acts + new_cognition_state`), distinct from Primary Output IR.
