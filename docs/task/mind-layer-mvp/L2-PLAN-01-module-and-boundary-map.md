# L2 Plan 01 - Module And Boundary Map

- Task Name: `mind-layer-mvp`
- Stage: `L2` / Part 01
- Date: 2026-02-08

## 1) Source File Map

### 1.1 New `src/mind` module layout

```text
src/mind/
├── mod.rs
├── error.rs
├── types.rs
├── state.rs
├── goal_manager.rs
├── preemption.rs
├── evaluator.rs
├── conflict.rs
├── evolution.rs
├── ports.rs
├── facade.rs
└── noop.rs
```

### 1.2 Responsibility map

1. `mod.rs`
- export stable Mind API surface
- hide internal helper utilities where possible

2. `error.rs`
- typed Mind-domain errors (`MindError`, `MindErrorKind`)
- no transport/runtime errors

3. `types.rs`
- shared domain types and identifiers
- commands, events, decisions, safe-point types

4. `state.rs`
- `MindState` definition and invariant-preserving transition helpers
- state snapshot/read model

5. `goal_manager.rs`
- goal lifecycle operations
- single-active-goal invariant enforcement

6. `preemption.rs`
- preemption domain and policy application
- safe-point and checkpoint token flow

7. `evaluator.rs`
- normative evaluation types + default deterministic evaluator

8. `conflict.rs`
- conflict case model and deterministic conflict resolver
- scoped conflict ownership only

9. `evolution.rs`
- evolution trigger and proposal model (proposal-only)

10. `ports.rs`
- trait ports:
  - `DelegationCoordinatorPort`
  - `MemoryPolicyPort`

11. `facade.rs`
- deterministic orchestrator loop
- command ingestion to typed output

12. `noop.rs`
- explicit no-op adapters for MVP:
  - `NoopDelegationCoordinator`
  - `NoopMemoryPolicy`

## 2) Test File Map

```text
tests/mind/
├── mod.rs
├── goal_manager.rs
├── preemption.rs
├── evaluator.rs
├── conflict.rs
├── evolution.rs
└── facade_loop.rs
```

Notes:

- mirror existing BDD naming style (`given_when_then`) in each test file.
- keep tests transport-free and deterministic.

## 3) Dependency Direction Rules

1. `src/mind/*` may depend only on:
- Rust std,
- same `src/mind/*` module,
- lightweight shared crates already in project (only if required).

2. `src/mind/*` must not depend on:
- `src/server.rs`
- `src/protocol.rs`
- `src/ai_gateway/*`
- socket, process, or network runtime logic.

3. `facade.rs` depends on policies and ports, not concrete helper/runtime adapters.

4. external implementation details enter only through traits in `ports.rs`.

## 4) Coupling Control Matrix

| Concern | Owner | Forbidden owner |
|---|---|---|
| Active-goal invariant | `goal_manager.rs` | `facade.rs` ad-hoc mutation |
| Continuity state storage | `state.rs` (`MindState`) | `facade.rs` fields as implicit cache |
| Preemption disposition | `preemption.rs` | `conflict.rs` |
| Memory remember/forget policy | `MemoryPolicyPort` | `evaluator.rs`, `conflict.rs` |
| Helper workload planning | `DelegationCoordinatorPort` | `goal_manager.rs` |
| Conflict ownership | `conflict.rs` scoped cases only | global policy logic |
| Evolution trigger | `evolution.rs` | transport/runtime adapters |

## 5) Public API Exposure (from `mind::mod`)

Expose minimal stable symbols only:

- `MindFacade`
- `MindState`
- `MindCommand`
- `MindCycleOutput`
- `MindEvent`
- `MindDecision`
- `PreemptionDisposition`
- `EvolutionDecision`
- `GoalManager`
- `MindError`
- `DelegationCoordinatorPort`
- `MemoryPolicyPort`

Everything else stays crate-private until needed.

## 6) Out-of-Scope for This Task

- protocol wiring into Unix socket runtime
- helper process execution runtime
- persistent memory store
- automatic body mutation/execution from evolution proposals
