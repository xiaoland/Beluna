# Cortex Module

Cortex is the stateless cognition module.

Code:
- `core/src/cortex/*`

Key properties:
1. Runtime boundary: `cortex(senses, physical_state, cognition_state) -> CortexOutput`.
2. No internal durable cognition store; persistence is delegated to Continuity by Cortex runtime.
3. Cognition state model is `goal_forest` with deterministic validation and rendering paths.
4. Primary runs as bounded AI Gateway thread turn loop and emits acts through structured tool calling.
5. Per-act wait semantics are integer seconds (`wait_for_sense_seconds`), not cycle-level bool.
6. Sense expansion uses unified `expand-senses` tool (`mode: raw | sub-agent`).
7. Sense delivery to Primary uses deterministic rendered lines:
- `- [monotonic-id]. [fq-sense-id]: [key=value,...]; [payload-truncated-if-needed]`.
8. Runtime wait path uses afferent deferral rule overwrite/reset during bounded wait windows.
9. `sleep` control is tool-driven (`ignore_all_trigger_for_seconds`) rather than Stem sleep act dispatch.

See:
- [Topography](./TOPOGRAPHY.md)
- [Topology Analysis](./TOPOLOGY_ANALYSIS.md)
