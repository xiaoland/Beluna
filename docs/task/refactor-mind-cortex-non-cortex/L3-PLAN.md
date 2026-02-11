# L3 Plan - Refactor Mind into Cortex + Non-Cortex (Index)
- Task Name: `refactor-mind-cortex-non-cortex`
- Stage: `L3` (implementation plan)
- Date: `2026-02-10`
- Status: `DRAFT_FOR_APPROVAL`
- Inputs: `L2` revised and accepted in conversation

This L3 package is split so execution can follow strict boundaries and checkpoints.

## File Map
1. `/Users/lanzhijiang/Development/Beluna/docs/task/refactor-mind-cortex-non-cortex/L3-PLAN-01-workstreams-and-sequence.md`
- ordered workstreams and dependency graph

2. `/Users/lanzhijiang/Development/Beluna/docs/task/refactor-mind-cortex-non-cortex/L3-PLAN-02-file-change-map.md`
- exact file add/modify/delete map

3. `/Users/lanzhijiang/Development/Beluna/docs/task/refactor-mind-cortex-non-cortex/L3-PLAN-03-core-pseudocode.md`
- cortex->non-cortex->spine cycle pseudocode

4. `/Users/lanzhijiang/Development/Beluna/docs/task/refactor-mind-cortex-non-cortex/L3-PLAN-04-ledger-settlement-and-debit-pipeline.md`
- reservation lifecycle, settlement idempotency, attribution-matched external debit flow

5. `/Users/lanzhijiang/Development/Beluna/docs/task/refactor-mind-cortex-non-cortex/L3-PLAN-05-test-execution-plan.md`
- contract-to-test mapping and execution commands

6. `/Users/lanzhijiang/Development/Beluna/docs/task/refactor-mind-cortex-non-cortex/L3-PLAN-06-doc-and-result-plan.md`
- docs migration and final `RESULT.md` layout

## Sub-Agent Reduction Strategy
No explicit sub-agent runtime is available here; cognitive load is reduced by:
1. isolating each implementation concern into one workstream/file,
2. enforcing checkpoint gates before moving to dependent workstreams.

## Stage Gate
Implementation starts only after explicit L3 approval.

Status: `READY_FOR_L3_REVIEW`
