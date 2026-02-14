# AGENTS.md for core/src/spine

Spine defines contracts for executing serial `ActDispatchRequest` and emitting settlement-linked ordered events.

## Invariants
- Spine accepts act dispatch requests only.
- Every event carries `reserve_entry_id` and `cost_attribution_id`.
- Event ordering is deterministic by Stem-provided `seq_no`.
- Routing is mechanical table lookup by route key.
