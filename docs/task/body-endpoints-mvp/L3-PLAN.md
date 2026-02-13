# L3 Plan - Body Endpoints MVP (Index)
- Task Name: `body-endpoints-mvp`
- Stage: `L3` (implementation plan)
- Date: `2026-02-12`
- Status: `DRAFT_FOR_APPROVAL`
- Inputs: accepted `L2` package (process-boundary architecture)

This L3 package is split into execution-focused files with strict checkpoints.

## File Map
1. `/Users/lanzhijiang/Development/Beluna/docs/task/body-endpoints-mvp/L3-PLAN-01-workstreams-and-sequence.md`
- ordered workstreams, dependencies, and stop/go gates.

2. `/Users/lanzhijiang/Development/Beluna/docs/task/body-endpoints-mvp/L3-PLAN-02-file-change-map.md`
- exact add/modify file list (no `core/src/*` edits).

3. `/Users/lanzhijiang/Development/Beluna/docs/task/body-endpoints-mvp/L3-PLAN-03-runtime-and-endpoint-pseudocode.md`
- `runtime/src` lifecycle and `std-body` endpoint-host pseudocode.

4. `/Users/lanzhijiang/Development/Beluna/docs/task/body-endpoints-mvp/L3-PLAN-04-socket-protocol-and-process-control.md`
- socket protocol handling, process supervision, reconnect behavior.

5. `/Users/lanzhijiang/Development/Beluna/docs/task/body-endpoints-mvp/L3-PLAN-05-test-execution-plan.md`
- contract-to-test mapping and command plan.

6. `/Users/lanzhijiang/Development/Beluna/docs/task/body-endpoints-mvp/L3-PLAN-06-doc-and-result-plan.md`
- documentation updates and final result output plan.

7. `/Users/lanzhijiang/Development/Beluna/docs/task/body-endpoints-mvp/L3a-PLAN.md`
- comprehensive step-by-step execution checklist to follow strictly.

## Sub-Agent Reduction Strategy
No explicit sub-agent runtime is available; cognitive load is reduced by:
1. splitting implementation into isolated workstreams with fixed gates,
2. front-loading protocol compatibility validation before writing endpoint runtime code.

## Stage Gate
Implementation starts only after explicit L3 approval.

Status: `READY_FOR_L3_REVIEW`
