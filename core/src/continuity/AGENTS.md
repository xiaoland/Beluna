# AGENTS.md for core/src/continuity

Continuity persists operational runtime state for Stem orchestration.

## Invariants
- Cognition state snapshot/persist must be deterministic.
- Capability patch/drop application follows arrival order.
- Dispatch gate decision contract is strictly `Continue` or `Break`.
- Spine event ingestion is deterministic and non-semantic.
