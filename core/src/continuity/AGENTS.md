# AGENTS.md for core/src/continuity

Continuity persists operational runtime state for Stem orchestration.

## Invariants
- Cognition state snapshot/persist must be deterministic.
- Capability patch/drop application follows arrival order.
- Dispatch gate decision contract is strictly `Continue` or `Break`.
- Continuity is store + guardrail only; cognition patch application stays inside Cortex.
- Continuity does not ingest Spine events or track act execution records.
- Continuity holds afferent-pathway sender for future sense emission capability.
- Cognition guardrails validate goal-forest node integrity (`numbering` nullable only for roots, parent linkage, `id`, `weight`, `status`, `summary`).
