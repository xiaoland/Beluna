# L3 Plan 01 - Workstreams And Sequence
- Task Name: `core-cortex-act-stem-refactor`
- Stage: `L3`
- Focus: execution order with hard gates
- Status: `DRAFT_FOR_APPROVAL`

## 1) Workstream WS0 - Preflight And Safety Gates
1. snapshot git status and confirm target branch.
2. ensure only planning docs are committed before implementation starts.
3. confirm unrelated local changes remain untouched.

Gate:
1. repository state understood and non-target changes isolated.

## 2) Workstream WS1 - Contract Cutover (Admission Removal + Runtime Types)
1. add `core/src/runtime_types.rs` and canonical shared types (`Sense`, `Act`, `CognitionState`, `PhysicalState`, `DispatchDecision`).
2. remove `core/src/admission/*`.
3. remove `pub mod admission` from `core/src/lib.rs`.
4. replace direct `admission::*` imports across `cortex`, `continuity`, `spine`, `body`.
5. compile check.

Gate:
1. no compile references to `crate::admission::*`.

## 3) Workstream WS2 - Stem Runtime Loop Introduction
1. add `core/src/ingress.rs` (bounded sender + ingress gate).
2. add `core/src/stem.rs` and implement sense-driven loop:
   - control-sense intercept,
   - compose state,
   - call cortex,
   - persist cognition,
   - inline serial act dispatch.
3. wire `core/src/main.rs` from `brainstem::run` to `stem::run`.
4. compile check.

Gate:
1. runtime starts with new stem entrypoint and no act queue.

## 4) Workstream WS3 - Module Adapters To New Interfaces
1. update cortex contracts to new function boundary outputting `Act[]`.
2. refactor continuity to:
   - persist cognition state,
   - expose capability patch/drop and snapshots,
   - serve continuity dispatch stage hooks.
3. refactor spine to single-act dispatch request path while preserving ordered event linkage fields.
4. adapt body/std endpoint invocation types.
5. compile check.

Gate:
1. end-to-end type graph stable under new contracts.

## 5) Workstream WS4 - Queue, Shutdown, And Wire Cutover
1. replace unbounded ingress with bounded MPSC usage in runtime.
2. implement signal handler policy:
   - close ingress gate,
   - block until `sleep` enqueued.
3. wire protocol changes:
   - remove admission-feedback envelope,
   - support `new_capabilities` and `drop_capabilities`.
4. config/schema updates removing obsolete loop batch fields.
5. compile check.

Gate:
1. shutdown path follows gate-then-block-sleep policy deterministically.

## 6) Workstream WS5 - Tests
1. delete admission tests.
2. add stem tests for loop/shutdown/capability patch/dispatch break behavior.
3. update cortex/continuity/spine/ledger tests to new contracts.
4. run targeted tests.
5. run full `cargo test`.

Gate:
1. targeted + full tests pass.

## 7) Workstream WS6 - Docs And Result Closure
1. update docs indexes and remove admission references.
2. update overview and glossary to new runtime model.
3. write `docs/task/RESULT.md` with implemented outputs and deviations.

Gate:
1. docs match implemented behavior and test evidence.

## 8) Sequence Constraints
1. WS1 must complete before WS2/WS3.
2. WS2 and WS3 can iterate in tandem after WS1 but merge only after compile green.
3. WS4 depends on WS2 runtime entry and WS3 interface stabilization.
4. WS5 depends on WS2-WS4 completion.
5. WS6 runs last.

Status: `READY_FOR_EXECUTION`
