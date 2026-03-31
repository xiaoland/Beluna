# L1 Plan Index - Cortex Loop Architecture
- Task Name: `cortex-loop-architecture`
- Stage: `L1` (high-level strategy)
- Date: `2026-03-01`
- Status: `DRAFT_FOR_APPROVAL`
- Source-of-truth policy: `tasks/*` remains procedural only; design choices are aligned with live code and authoritative docs under `docs/modules/*` and contracts.

## L1 Objective
Define a concrete architecture strategy for each micro-task so L2 can focus on interfaces and algorithms, not ownership debates.

## Cross-Cutting Architecture Decisions
1. Runtime is split into two independent loops:
- Stem runtime is clock authority and pathway factory.
- Cortex runtime is hybrid-driven and never directly invoked by Stem.
2. Dependency Injection is composition-root only:
- `main()` creates `AppContext` and wires modules.
- Runtime modules receive narrow ports/handles, not the whole context.
3. Pathway ownership and runtime consumption are separated:
- Pathway modules live under Stem.
- Cortex owns afferent receive handle and consumes senses in its own runtime.
4. Contract hard-cuts are allowed:
- no backward compatibility shim for sense payload migration.
5. Determinism is mandatory for rendered cognition inputs:
- sense metadata rendering and goal-forest input rendering must be deterministic.

## Maintainability-First Refactor Proposals
1. Introduce explicit runtime modules:
- `core/src/stem/runtime.rs`
- `core/src/stem/pathways/afferent.rs`
- `core/src/stem/pathways/efferent.rs`
- `core/src/cortex/runtime_loop.rs`
2. Introduce a `ports` layer for cross-module contracts (`ClockPort`, `AfferentControlPort`, `CognitionPersistencePort`, `EfferentProducerPort`) to reduce concrete coupling.
3. Keep `AppContext` as a wiring object only; forbid business logic methods on it.
4. Replace ad-hoc helper parsing paths with shared deterministic formatter/parser modules.

## Micro-Task L1 Plans
1. [L1-PLAN-01-runtime-skeleton-and-app-context-di.md](/Users/lanzhijiang/Development/Beluna/tasks/cortex-loop-architecture/L1-PLAN-01-runtime-skeleton-and-app-context-di.md)
2. [L1-PLAN-02-sense-model-and-wire-migration.md](/Users/lanzhijiang/Development/Beluna/tasks/cortex-loop-architecture/L1-PLAN-02-sense-model-and-wire-migration.md)
3. [L1-PLAN-03-afferent-deferral-engine-and-sidecar.md](/Users/lanzhijiang/Development/Beluna/tasks/cortex-loop-architecture/L1-PLAN-03-afferent-deferral-engine-and-sidecar.md)
4. [L1-PLAN-04-cortex-primary-tooling-and-act-emission.md](/Users/lanzhijiang/Development/Beluna/tasks/cortex-loop-architecture/L1-PLAN-04-cortex-primary-tooling-and-act-emission.md)
5. [L1-PLAN-05-goal-forest-reset-and-thread-rewrite.md](/Users/lanzhijiang/Development/Beluna/tasks/cortex-loop-architecture/L1-PLAN-05-goal-forest-reset-and-thread-rewrite.md)
6. [L1-PLAN-06-state-ownership-and-continuity-refactor.md](/Users/lanzhijiang/Development/Beluna/tasks/cortex-loop-architecture/L1-PLAN-06-state-ownership-and-continuity-refactor.md)
7. [L1-PLAN-07-efferent-fifo-serial-pipeline.md](/Users/lanzhijiang/Development/Beluna/tasks/cortex-loop-architecture/L1-PLAN-07-efferent-fifo-serial-pipeline.md)
8. [L1-PLAN-08-docs-contracts-refresh.md](/Users/lanzhijiang/Development/Beluna/tasks/cortex-loop-architecture/L1-PLAN-08-docs-contracts-refresh.md)

## Recommended Execution Order
1. `01` runtime skeleton and DI boundary
2. `02` sense model hard-cut
3. `03` afferent deferral engine
4. `04` structured act tooling and bounded waiting
5. `05` goal-forest reset transaction
6. `06` state ownership and continuity split
7. `07` efferent FIFO extraction
8. `08` docs/contracts/result consolidation

## L1 Exit
1. Each micro-task now has a high-level architecture strategy.
2. Cross-task coupling and dependency order are explicit.
3. Ready for L1 review and approval before drafting L2 files.

Status: `READY_FOR_L1_APPROVAL`
