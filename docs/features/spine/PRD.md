# Spine PRD

## Purpose

Spine is the always-on control substrate between Non-cortex and body endpoints.

For MVP scope here, Spine defines execution contracts only.

## Requirements

- Spine accepts `AdmittedAction[]` only.
- Execution report mode is explicit:
  - `BestEffortReplayable`
  - `SerializedDeterministic`
- Settlement events include `reserve_entry_id` and `cost_attribution_id`.
- Events are totally ordered by `seq_no` and replayable by cursor.
