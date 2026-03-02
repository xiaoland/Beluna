# L0 Plan 07 - Efferent FIFO Serial Pipeline
- Task: `cortex-loop-architecture`
- Micro-task: `07-efferent-fifo-serial-pipeline`
- Stage: `L0`
- Date: `2026-03-01`
- Status: `DRAFT_FOR_APPROVAL`

## Objective
Extract a first-class efferent FIFO pathway that preserves current serial dispatch semantics.

## Scope
1. Create `efferent_pathway` under Stem module ownership, instantiated by Stem runtime and injected via AppContext handles.
2. Cortex publishes acts to efferent FIFO handle.
3. Serial consumers are fixed:
- Continuity stage
- then Spine stage.
4. Preserve existing deterministic dispatch outcomes and status behavior.
5. Integrate optional `act_instance_id` correlation into resulting senses.

## Current State
1. Serial behavior exists, but embedded in Stem dispatch worker.
2. No standalone efferent pathway abstraction or handle contract.

## Target State
1. Efferent behavior is encapsulated and reusable.
2. Continuity->Spine ordering remains invariant.
3. Backpressure and shutdown semantics are explicit.

## Key Gaps
1. New pathway API and lifecycle management.
2. Migration of dispatch worker responsibilities from Stem into pathway.
3. Clear handoff between Cortex runtime and efferent pathway producer handle.
4. Dependency wiring contract between AppContext composition root and consumer stages.

## Risks
1. Queue ownership mistakes can reorder or drop acts.
2. Shutdown race can lose in-flight acts without clear drain policy.

## L0 Exit Criteria
1. Efferent FIFO API and consumer chain are contractually explicit.
2. Dispatch ordering invariants are preserved.
3. Shutdown/drain behavior is defined.
