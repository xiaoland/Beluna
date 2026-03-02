# L0 Plan Index - Cortex Loop Architecture
- Task Name: `cortex-loop-architecture`
- Stage: `L0` (index)
- Date: `2026-03-01`
- Status: `DRAFT_FOR_APPROVAL`
- Source-of-truth policy: `docs/task/*` is not used as architecture truth; these plans are derived from live code + `docs/modules/*` + `docs/contracts/*`.

## Locked Decisions
1. Cortex loop is hybrid-driven (time + event: sense/act).
2. Clock authority stays in Stem.
3. Cortex must never be invoked by Stem; Cortex runs in its own thread/task.
4. Pathways are Stem-owned submodules, instantiated by Stem runtime, not by `main()`.
5. Runtime wiring uses Dependency Injection with `AppContext` as composition root.
- `AppContext` is created in `main()`.
- It owns runtime instances/handles (no global static pathway singletons).
- Modules receive narrow dependency bundles/ports, not the full context.
6. Afferent:
- deferral-only policy
- rules use `min_weight` + `fq-sense-id` regex
- rule control is overwrite/reset only
- deferred FIFO buffer with `max_deferring_nums` oldest eviction + warning log
- observe-only sidecar
- Cortex Primary tool can directly call afferent control API.
7. Efferent:
- FIFO queue
- serial pipeline `Continuity -> Spine`.
8. Act emission:
- structured Primary tool-calling
- remove prompt-based act dispatch path
- remove act descriptor catalog from system prompt
- per-act `payload` + bounded integer `wait_for_sense`.
9. Goal forest patch:
- `reset: bool` with thread-history rewrite and system prompt refresh/resend.
10. Sense model:
- immediate payload migration to text
- remove `metadata` field
- add `weight ∈ [0,1]` default `0`
- add optional `act_instance_id`
- Primary render metadata as deterministic `key=value` list.
11. State refactor:
- `Arc<RwLock<PhysicalState>>` shared state
- Cortex calls Continuity directly for persistence
- remove Stem cognition persistence logic
- remove L1 memory
- goal forest helper is a single shared instance managed by `AppContext` with deterministic `to_input_ir`.

## Micro-Task L0 Plans
1. [L0-PLAN-01-runtime-skeleton-and-app-context-di.md](/Users/lanzhijiang/Development/Beluna/docs/task/cortex-loop-architecture/L0-PLAN-01-runtime-skeleton-and-app-context-di.md)
2. [L0-PLAN-02-sense-model-and-wire-migration.md](/Users/lanzhijiang/Development/Beluna/docs/task/cortex-loop-architecture/L0-PLAN-02-sense-model-and-wire-migration.md)
3. [L0-PLAN-03-afferent-deferral-engine-and-sidecar.md](/Users/lanzhijiang/Development/Beluna/docs/task/cortex-loop-architecture/L0-PLAN-03-afferent-deferral-engine-and-sidecar.md)
4. [L0-PLAN-04-cortex-primary-tooling-and-act-emission.md](/Users/lanzhijiang/Development/Beluna/docs/task/cortex-loop-architecture/L0-PLAN-04-cortex-primary-tooling-and-act-emission.md)
5. [L0-PLAN-05-goal-forest-reset-and-thread-rewrite.md](/Users/lanzhijiang/Development/Beluna/docs/task/cortex-loop-architecture/L0-PLAN-05-goal-forest-reset-and-thread-rewrite.md)
6. [L0-PLAN-06-state-ownership-and-continuity-refactor.md](/Users/lanzhijiang/Development/Beluna/docs/task/cortex-loop-architecture/L0-PLAN-06-state-ownership-and-continuity-refactor.md)
7. [L0-PLAN-07-efferent-fifo-serial-pipeline.md](/Users/lanzhijiang/Development/Beluna/docs/task/cortex-loop-architecture/L0-PLAN-07-efferent-fifo-serial-pipeline.md)
8. [L0-PLAN-08-docs-contracts-refresh.md](/Users/lanzhijiang/Development/Beluna/docs/task/cortex-loop-architecture/L0-PLAN-08-docs-contracts-refresh.md)

## Dependency Order
1. `01` runtime skeleton and AppContext DI lifecycle
2. `02` sense schema migration
3. `03` afferent deferral engine
4. `04` primary tooling and structured act emission
5. `05` goal forest reset thread rewrite
6. `06` state ownership and persistence refactor
7. `07` efferent FIFO extraction/integration
8. `08` docs/contracts/RESULT synchronization

## Maintainability Checkpoint
Immediate text payload migration and AppContext overuse can reduce maintainability if boundaries are not strict.

Mitigation baseline is required across micro-tasks:
1. strict composition-root lifecycle for AppContext-owned resources
2. narrow typed dependency bundles (`StemDeps`, `CortexDeps`, `PathwayHandles`) instead of passing entire `AppContext`
3. deterministic formatter/parser tests for key=value metadata rendering
4. bounded wait, bounded deferral, and eviction observability metrics/logs.

## L0 Exit
1. Task has been decomposed into 8 micro-tasks, each with its own L0 plan.
2. Current file acts as index and decision ledger.
3. Ready to proceed to L1 per micro-task after your approval.

Status: `READY_FOR_L0_APPROVAL`
