# L3 Plan 05 - Test Execution Plan
- Task Name: `core-cortex-act-stem-refactor`
- Stage: `L3`
- Focus: contract-to-test execution and command order
- Status: `DRAFT_FOR_APPROVAL`

## 1) Test Phases
1. Phase A: compile integrity after contract cutover.
2. Phase B: targeted module tests.
3. Phase C: stem integration tests.
4. Phase D: full regression suite.

## 2) Phase A - Compile Gates
1. `cd /Users/lanzhijiang/Development/Beluna/core && cargo check`
2. if fail: fix unresolved references before running tests.

## 3) Phase B - Targeted Tests
1. Cortex:
   - `cd /Users/lanzhijiang/Development/Beluna/core && cargo test cortex:: -- --nocapture`
2. Continuity:
   - `cd /Users/lanzhijiang/Development/Beluna/core && cargo test continuity:: -- --nocapture`
3. Spine:
   - `cd /Users/lanzhijiang/Development/Beluna/core && cargo test spine:: -- --nocapture`
4. Ledger:
   - `cd /Users/lanzhijiang/Development/Beluna/core && cargo test ledger:: -- --nocapture`

Gate:
1. targeted suites green before stem integration run.

## 4) Phase C - Stem Integration
1. `cd /Users/lanzhijiang/Development/Beluna/core && cargo test stem:: -- --nocapture`
2. key assertions:
   - no act queue, serial inline dispatch,
   - `Break` aborts current act only,
   - capability patch/drop applied before same-cycle cortex call,
   - shutdown gate + blocking sleep enqueue path.

Gate:
1. stem behavior contracts validated.

## 5) Phase D - Full Suite
1. `cd /Users/lanzhijiang/Development/Beluna/core && cargo test -- --nocapture`

Gate:
1. no regressions outside removed admission surfaces.

## 6) Failure Handling Policy
1. If unrelated pre-existing failures appear:
   - record them,
   - do not expand scope to fix unrelated issues.
2. If failures are caused by admission deletion:
   - migrate tests or remove obsolete assertions.

## 7) Contract Matrix (Must-Pass)
1. Queue backpressure is blocking (no drop-oldest).
2. Shutdown ingress gate blocks new senses then enqueues sleep.
3. Cortex is not called on `Sense::Sleep`.
4. `new_capabilities`/`drop_capabilities` change same-cycle physical state.
5. Dispatch stage order is Ledger -> Continuity -> Spine.
6. `DispatchDecision::Break` applies only to current act.

Status: `READY_FOR_EXECUTION`
