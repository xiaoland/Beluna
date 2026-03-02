# L2 Plan Index - Cortex Loop Architecture
- Task Name: `cortex-loop-architecture`
- Stage: `L2` (low-level design)
- Date: `2026-03-02`
- Status: `DRAFT_FOR_APPROVAL`
- Source-of-truth policy: `docs/task/*` is procedural only; final architecture truth remains in code and `docs/modules/*`.

## L2 Objective
Freeze per-micro-task interfaces, ownership boundaries, data structures, and deterministic algorithms before implementation planning.

## L2 Decisions Locked So Far
1. `02` hard-cuts `Sense` to struct-only and removes control senses.
2. `03` introduces pathway-owned rule-control port with single-rule overwrite + full reset semantics.
3. `03` places afferent pathway under Stem module ownership (`stem::afferent_pathway`).

## Micro-Task L2 Plans
1. [L2-PLAN-01-runtime-skeleton-and-app-context-di.md](/Users/lanzhijiang/Development/Beluna/docs/task/cortex-loop-architecture/L2-PLAN-01-runtime-skeleton-and-app-context-di.md)
2. [L2-PLAN-02-sense-model-and-wire-migration.md](/Users/lanzhijiang/Development/Beluna/docs/task/cortex-loop-architecture/L2-PLAN-02-sense-model-and-wire-migration.md)
3. [L2-PLAN-03-afferent-deferral-engine-and-sidecar.md](/Users/lanzhijiang/Development/Beluna/docs/task/cortex-loop-architecture/L2-PLAN-03-afferent-deferral-engine-and-sidecar.md)
4. [L2-PLAN-04-cortex-primary-tooling-and-act-emission.md](/Users/lanzhijiang/Development/Beluna/docs/task/cortex-loop-architecture/L2-PLAN-04-cortex-primary-tooling-and-act-emission.md)
5. [L2-PLAN-05-goal-forest-reset-and-thread-rewrite.md](/Users/lanzhijiang/Development/Beluna/docs/task/cortex-loop-architecture/L2-PLAN-05-goal-forest-reset-and-thread-rewrite.md)
6. [L2-PLAN-06-state-ownership-and-continuity-refactor.md](/Users/lanzhijiang/Development/Beluna/docs/task/cortex-loop-architecture/L2-PLAN-06-state-ownership-and-continuity-refactor.md)
7. [L2-PLAN-07-efferent-fifo-serial-pipeline.md](/Users/lanzhijiang/Development/Beluna/docs/task/cortex-loop-architecture/L2-PLAN-07-efferent-fifo-serial-pipeline.md)
8. [L2-PLAN-08-docs-contracts-refresh.md](/Users/lanzhijiang/Development/Beluna/docs/task/cortex-loop-architecture/L2-PLAN-08-docs-contracts-refresh.md)

## L2 Exit
1. Micro-task L2 contracts are reviewable and implementation-ready.
2. Ready to proceed to L3 implementation plans per approved micro-task.

Status: `READY_FOR_L2_APPROVAL`
