# Spine Contracts

Boundary:
1. input: `AdmittedActionBatch`
2. output: `SpineExecutionReport`

Must hold:
1. admitted-action-only execution contract
2. explicit execution mode semantics
3. ordered/replayable events by `seq_no`
4. settlement linkage via `reserve_entry_id` and `cost_attribution_id`
5. deterministic per-action rejection mapping for route miss and endpoint error

Routing semantics:
1. route key is composite of `affordance_key` and `capability_handle`
2. routing behavior is mechanical table lookup (no transport logic)
