# AGENTS.md for core/src/spine

Spine defines interface/contracts for executing admitted actions and emitting ordered settlement events.

## Invariants
- Spine accepts admitted actions only.
- Settlement events carry `reserve_entry_id` and `cost_attribution_id`.
- Event streams are totally ordered by `seq_no` and replayable by cursor.
