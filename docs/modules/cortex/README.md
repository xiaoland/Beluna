# Cortex Module

Cortex is the stateless cognition module.

Code:
- `core/src/cortex/*`

Key properties:
1. Runtime boundary: `cortex(senses, physical_state, cognition_state) -> CortexOutput`.
2. No internal durable cognition store; persistence is delegated to Continuity by Cortex runtime.
3. Cognition state model is `goal_forest` with deterministic validation and rendering paths.
4. Primary runs as bounded AI Gateway thread turn loop and emits acts through structured tool calling.
5. Per-act wait semantics are integer ticks (`wait_for_sense`), not cycle-level bool.
6. Sense expansion uses unified `expand-senses` tool with direct `tasks[]` arguments and per-task optional `use_subagent_and_instruction_is`.
7. Sense delivery to Primary uses deterministic rendered lines:
- `- [monotonic-id]. endpoint_id=[endpoint_id], sense_id=[sense_id], weight=[weight][, truncated_ratio=[0..1 if truncated]]; payload="[payload-truncated-if-needed]"`.
8. Runtime wait path skips admitted ticks until matching correlated senses are buffered or wait ticks expire; no afferent deferral rule mutation is used.
9. `sleep` control is tool-driven (`ignore_all_trigger_for_ticks`) rather than Stem sleep act dispatch.

See:
- [Topography](./TOPOGRAPHY.md)
- [Topology Analysis](./TOPOLOGY_ANALYSIS.md)
- [Sequence](./SEQUENCE.md)
