# Cortex Contracts

Related:
- Goal forest detailed contract and manual testing checklist: `./goal-forest.md`

Boundary:
1. Input: `Sense[]`, `PhysicalState`, `CognitionState`.
2. Output: `CortexOutput { emitted_acts, new_cognition_state, control }`.

Contract details:
1. `CortexOutput.emitted_acts[]` contains `EmittedAct` entries:
- `act: Act`
- `wait_for_sense_seconds: u64` (`0` means no wait)
- `expected_fq_sense_ids: Vec<String>` (optional expected correlated sense ids).
2. `CortexOutput.control` currently supports `ignore_all_trigger_for_seconds`.

Must hold:
1. Cortex is stateless at runtime boundary and has no direct side effects on Stem/Spine.
2. `Act` is non-binding; execution decisions remain in the efferent pipeline.
3. Primary runs as bounded internal micro-loop (`max_internal_steps`) on AI Gateway chat thread.
4. Primary tool calls are deterministic by contract surface and include:
- dedicated per-act tools (tool name aliases mapped to fq act ids)
- `expand-senses`
- `patch-goal-forest`
- `overwrite-sense-deferral-rule`
- `reset-sense-deferral-rules`
- `sleep`.
5. Sense expansion tool contract is unified:
- `mode: raw | sub-agent`
- `senses_to_expand[].sense_id` uses rendered id format: `"<monotonic-id>. <fq-sense-id>"`.
6. Primary failure/timeout/exhaustion degrades to noop with unchanged cognition state.
7. Goal-forest mutations are applied via `patch-goal-forest` tool calls, not output tags.
8. Proprioception is rendered in Input IR and must be refreshed from physical state before each Primary turn dispatch.
