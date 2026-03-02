# L1 Plan 07 - Efferent FIFO Serial Pipeline
- Task: `cortex-loop-architecture`
- Micro-task: `07-efferent-fifo-serial-pipeline`
- Stage: `L1`
- Date: `2026-03-01`
- Status: `DRAFT_FOR_APPROVAL`

## High-Level Strategy
1. Extract act dispatch into a dedicated efferent pathway module with explicit FIFO queue ownership.
2. Preserve current serial consumer order: `Continuity -> Spine`.
3. Keep Cortex as producer only; no direct dispatch to Continuity or Spine.
4. Define backpressure and shutdown drain policy explicitly.

## Architectural Design
1. Efferent pathway components:
- producer handle for Cortex
- bounded FIFO queue
- single serial consumer worker.
2. Dispatch pipeline per act:
- Continuity middleware stage (`Continue` or `Break`)
- Spine final execution stage
- terminal status/feedback emission.
3. Correlation:
- preserve and propagate `act_instance_id`
- emitted senses can carry optional correlated `act_instance_id`.
4. Shutdown behavior:
- stop intake
- drain inflight queue under bounded deadline
- emit telemetry for dropped acts on timeout.

## Key Technical Decisions
1. Ordering guarantee is queue-order strict for all accepted acts.
2. Continuity `Break` short-circuits only current act, not queue progression.
3. Queue capacity and retry behavior are configuration-backed and observable.
4. Efferent module is owned by Stem namespace but runtime-accessed through DI handles.

## Dependency Requirements
1. Micro-task `01` runtime split and handles are required first.
2. Micro-task `04` structured act output should be stable before final routing cutover.
3. Micro-task `06` continuity boundary changes must be aligned for dispatch context schema.

## L1 Exit Criteria
1. FIFO + serial ordering invariants are explicit.
2. Producer/consumer boundaries are decoupled and DI-friendly.
3. Backpressure and shutdown drain semantics are defined.
