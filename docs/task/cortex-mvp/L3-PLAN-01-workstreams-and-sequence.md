# L3-01 - Workstreams And Sequence
- Task Name: `cortex-mvp`
- Stage: `L3` detail: execution sequence
- Date: `2026-02-11`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Execution Principles
1. Cortex is reactor-only; old `step` API is removed from canonical path.
2. Cortex is stateless for durable goal/commitment storage.
3. Reactor progression is inbox-event driven only.
4. `IntentAttempt` remains non-binding and world-relative.
5. `IntentAttempt` and feedback must correlate by `attempt_id`.
6. `IntentAttempt` must include `based_on: [sense_id...]`.
7. Business output purity is preserved (`ReactionResult` has no telemetry payload).
8. One-primary/N-subcall/one-repair/noop fallback bounds are hard constraints.

## 2) Ordered Workstreams
### Workstream A - Reactor Contract Cutover
Scope:
1. replace command-step contracts with `ReactionInput`/`ReactionResult`.
2. update `cortex/mod.rs` exports to reactor surfaces.
3. retire old command/commitment-centric public API.

Exit criteria:
1. cortex module compiles with reactor contracts as canonical types.
2. no compile references to `CortexFacade::step` remain in active paths.

### Workstream B - Ports + Clamp Foundation
Scope:
1. implement primary/extractor/filler async ports.
2. implement deterministic clamp module.
3. enforce `attempt_id` + `based_on` + schema/catalog/cap bounds.

Exit criteria:
1. clamp can convert drafts to valid attempts deterministically.
2. invalid drafts are dropped with deterministic violation records.

### Workstream C - Reactor Engine + One-Repair Pipeline
Scope:
1. implement `CortexReactor::run` and `react_once`.
2. enforce cycle call/token/time limits.
3. implement single repair branch and noop fallback.

Exit criteria:
1. reactor loop advances one cycle per inbox input.
2. boundedness assertions pass in unit tests.

### Workstream D - AI Gateway Runtime Adapters
Scope:
1. implement ai-gateway-backed primary adapter.
2. implement ai-gateway-backed extractor and filler adapters.
3. enforce capability preconditions and graceful noop on mismatch.

Exit criteria:
1. runtime adapters call `AIGateway::infer_once` with expected limits.
2. adapter failures are mapped to cycle-local noop outcomes.

### Workstream E - Runtime Ingress + Protocol + Backpressure
Scope:
1. extend protocol event types for sense/env snapshot/admission feedback inputs.
2. implement `CortexIngressAssembler` in server boundary.
3. wire bounded inbox/outbox channels and reactor task lifecycle.

Exit criteria:
1. ingress produces bounded `ReactionInput` only.
2. upstream backpressure remains mechanical (no semantic overrides).

### Workstream F - Tests
Scope:
1. add new reactor/clamp/adapter test suites.
2. update integration flow tests for feedback `attempt_id` correlation.
3. remove or retire step/planner-centric cortex tests.

Exit criteria:
1. all new cortex contract tests pass.
2. no network dependency in unit tests.

### Workstream G - Docs + Result
Scope:
1. update cortex feature/module/contracts docs to reactor semantics.
2. align overview and AGENTS state notes.
3. write task result report.

Exit criteria:
1. docs and contracts reflect implemented behavior.
2. `docs/task/cortex-mvp/RESULT.md` is complete.

## 3) Dependency Graph
1. A -> B
2. B -> C
3. C -> D
4. C + D -> E
5. E -> F
6. F -> G

## 4) Stop/Go Checkpoints
1. After A: reactor contracts compile and old step API is not canonical.
2. After B: clamp invariants and `based_on` requirements are test-covered.
3. After C: one-primary/N-subcall/one-repair bounds are enforced by tests.
4. After D: adapter path runs with mock substitution in tests.
5. After E: server loop handles ingress and backpressure without protocol regressions.
6. After F: full `cargo test` passes.

## 5) Out Of Scope
1. Persistent goal store inside Cortex.
2. Semantic interpretation inside Admission/Continuity.
3. Multi-repair or unbounded retry logic.

Status: `READY_FOR_L3_REVIEW`
