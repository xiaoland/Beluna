# Spine HLD

## Interface

Input:
- `AdmittedActionBatch`

Output:
- `SpineExecutionReport { mode, events[], replay_cursor }`

## Event Model

- `ActionApplied`
- `ActionRejected`
- `ActionDeferred`

Each settlement-capable event references reservation linkage fields for non-cortex reconciliation.
