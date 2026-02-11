# Spine Contracts

Boundary:
- input: `AdmittedActionBatch`
- output: `SpineExecutionReport`

Must hold:
- admitted-action-only execution contract
- explicit execution mode semantics
- ordered/replayable events
- settlement linkage via `reserve_entry_id` and `cost_attribution_id`
