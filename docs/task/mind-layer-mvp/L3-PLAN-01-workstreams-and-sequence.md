# L3-01 - Workstreams And Sequence

- Task Name: `mind-layer-mvp`
- Stage: `L3` detail: execution sequence
- Date: 2026-02-09
- Status: `DRAFT_FOR_APPROVAL`

## 1) Execution Principles

1. Mind stays transport-agnostic: no Unix socket/protocol coupling.
2. Keep all state in explicit `MindState`; facade is orchestrator only.
3. Enforce single-active-goal and deterministic transitions through `GoalManager`.
4. Implement policy ports (`DelegationCoordinatorPort`, `MemoryPolicyPort`) before wiring loop logic.
5. Add tests per workstream before moving forward.

## 2) Ordered Workstreams

### Workstream A - Mind Foundation Scaffolding

Scope:

- add `src/mind/*` module skeleton,
- add typed Mind errors and shared core types,
- expose module from `src/lib.rs`.

Exit criteria:

- module compiles,
- no forbidden dependencies on runtime/socket/ai_gateway.

### Workstream B - State + Goal Manager + Preemption Core

Scope:

- implement `MindState`,
- implement goal lifecycle operations in `GoalManager`,
- implement safe point model and preemption decision types.

Exit criteria:

- single-active-goal invariant enforced,
- preemption dispositions represented with safe point/checkpoint fields.

### Workstream C - Policy Surfaces And Resolvers

Scope:

- implement trait ports in `ports.rs`,
- add no-op policy adapters in `noop.rs`,
- implement evaluator skeleton, conflict resolver, evolution decider.

Exit criteria:

- conflict ownership boundaries explicit in code,
- evolution is proposal-only type path.

### Workstream D - Deterministic Facade Loop

Scope:

- implement `MindFacade::step(MindCommand)` deterministic loop,
- wire preemption -> delegation -> evaluation -> conflict -> memory policy -> evolution order,
- emit typed `MindEvent` and `MindDecision`.

Exit criteria:

- loop sequence is deterministic,
- no side-effect coupling outside ports.

### Workstream E - Tests And Contract Alignment

Scope:

- add `tests/mind/*` unit tests with BDD naming,
- add contract-focused assertions for invariants and policy behavior,
- add determinism tests for repeated identical inputs.

Exit criteria:

- all new Mind tests pass,
- existing tests remain green.

### Workstream F - Docs And Task Result

Scope:

- add mind feature/module/contract docs,
- update top-level docs indexes,
- write `docs/task/mind-layer-mvp/RESULT.md` with evidence.

Exit criteria:

- docs are linked from indexes,
- result document reflects implementation and test evidence.

## 3) Dependency Graph

1. A -> B
2. A -> C
3. B + C -> D
4. D -> E
5. E -> F

## 4) Stop/Go Checkpoints

1. After A: verify no boundary leaks to server/protocol/gateway modules.
2. After B: verify active-goal invariants and valid state transitions.
3. After C: verify conflict ownership scope and no-op ports compile.
4. After D: verify deterministic loop ordering against L2.
5. After E: verify no regression in existing project tests.

## 5) Out-of-Scope For This Implementation

1. Unix socket protocol integration for Mind.
2. Real helper process execution runtime.
3. Persistent memory storage system.
4. Automatic execution of evolution proposals.

Status: `READY_FOR_L3_REVIEW`
