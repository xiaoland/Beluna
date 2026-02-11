# L3-02 - File Change Map
- Task Name: `refactor-mind-cortex-non-cortex`
- Stage: `L3` detail: file-level execution map
- Date: `2026-02-10`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Files To Delete
1. `/Users/lanzhijiang/Development/Beluna/core/src/mind/mod.rs`
2. `/Users/lanzhijiang/Development/Beluna/core/src/mind/error.rs`
3. `/Users/lanzhijiang/Development/Beluna/core/src/mind/types.rs`
4. `/Users/lanzhijiang/Development/Beluna/core/src/mind/state.rs`
5. `/Users/lanzhijiang/Development/Beluna/core/src/mind/goal_manager.rs`
6. `/Users/lanzhijiang/Development/Beluna/core/src/mind/preemption.rs`
7. `/Users/lanzhijiang/Development/Beluna/core/src/mind/evaluator.rs`
8. `/Users/lanzhijiang/Development/Beluna/core/src/mind/conflict.rs`
9. `/Users/lanzhijiang/Development/Beluna/core/src/mind/evolution.rs`
10. `/Users/lanzhijiang/Development/Beluna/core/src/mind/ports.rs`
11. `/Users/lanzhijiang/Development/Beluna/core/src/mind/facade.rs`
12. `/Users/lanzhijiang/Development/Beluna/core/src/mind/noop.rs`
13. `/Users/lanzhijiang/Development/Beluna/core/src/mind/AGENTS.md` (or rewrite to new module guides if kept)
14. `/Users/lanzhijiang/Development/Beluna/core/tests/mind/*`
15. `/Users/lanzhijiang/Development/Beluna/core/tests/mind_bdt.rs`

## 2) Files To Add - Cortex
1. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/mod.rs`
2. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/error.rs`
3. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/types.rs`
4. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/state.rs`
5. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/commitment_manager.rs`
6. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/planner.rs`
7. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/facade.rs`
8. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/ports.rs`
9. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/noop.rs`
10. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/AGENTS.md`

## 3) Files To Add - Non-Cortex
1. `/Users/lanzhijiang/Development/Beluna/core/src/non_cortex/mod.rs`
2. `/Users/lanzhijiang/Development/Beluna/core/src/non_cortex/error.rs`
3. `/Users/lanzhijiang/Development/Beluna/core/src/non_cortex/types.rs`
4. `/Users/lanzhijiang/Development/Beluna/core/src/non_cortex/continuity.rs`
5. `/Users/lanzhijiang/Development/Beluna/core/src/non_cortex/affordance.rs`
6. `/Users/lanzhijiang/Development/Beluna/core/src/non_cortex/resolver.rs`
7. `/Users/lanzhijiang/Development/Beluna/core/src/non_cortex/ledger.rs`
8. `/Users/lanzhijiang/Development/Beluna/core/src/non_cortex/debit_sources.rs`
9. `/Users/lanzhijiang/Development/Beluna/core/src/non_cortex/facade.rs`
10. `/Users/lanzhijiang/Development/Beluna/core/src/non_cortex/ports.rs`
11. `/Users/lanzhijiang/Development/Beluna/core/src/non_cortex/noop.rs`
12. `/Users/lanzhijiang/Development/Beluna/core/src/non_cortex/AGENTS.md`

## 4) Files To Add - Spine
1. `/Users/lanzhijiang/Development/Beluna/core/src/spine/mod.rs`
2. `/Users/lanzhijiang/Development/Beluna/core/src/spine/error.rs`
3. `/Users/lanzhijiang/Development/Beluna/core/src/spine/types.rs`
4. `/Users/lanzhijiang/Development/Beluna/core/src/spine/ports.rs`
5. `/Users/lanzhijiang/Development/Beluna/core/src/spine/noop.rs`
6. `/Users/lanzhijiang/Development/Beluna/core/src/spine/AGENTS.md`

## 5) Files To Modify - Core Shared Surfaces
1. `/Users/lanzhijiang/Development/Beluna/core/src/lib.rs`
- remove `pub mod mind;`
- add `pub mod cortex; pub mod non_cortex; pub mod spine;`

2. `/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/types.rs`
- add optional `cost_attribution_id` to request and event/telemetry-relevant types.

3. `/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/telemetry.rs`
- include attribution field where needed for debit-source correlation.

4. `/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/gateway.rs`
- propagate attribution through request lifecycle and telemetry emission.

## 6) Files To Add - Tests
1. `/Users/lanzhijiang/Development/Beluna/core/tests/cortex/mod.rs`
2. `/Users/lanzhijiang/Development/Beluna/core/tests/cortex/*.rs`
3. `/Users/lanzhijiang/Development/Beluna/core/tests/non_cortex/mod.rs`
4. `/Users/lanzhijiang/Development/Beluna/core/tests/non_cortex/*.rs`
5. `/Users/lanzhijiang/Development/Beluna/core/tests/spine/mod.rs`
6. `/Users/lanzhijiang/Development/Beluna/core/tests/spine/contracts.rs`
7. `/Users/lanzhijiang/Development/Beluna/core/tests/cortex_non_cortex_flow.rs`

## 7) Files To Add/Modify - Docs
1. add `docs/features/{cortex,non-cortex,spine}/*`
2. add `docs/modules/{cortex,non-cortex,spine}/*`
3. add `docs/contracts/{cortex,non-cortex,spine}/*`
4. modify index files:
- `/Users/lanzhijiang/Development/Beluna/docs/features/README.md`
- `/Users/lanzhijiang/Development/Beluna/docs/modules/README.md`
- `/Users/lanzhijiang/Development/Beluna/docs/contracts/README.md`
- `/Users/lanzhijiang/Development/Beluna/docs/product/overview.md`
- `/Users/lanzhijiang/Development/Beluna/docs/product/glossary.md`

## 8) Task Result File
1. `/Users/lanzhijiang/Development/Beluna/docs/task/refactor-mind-cortex-non-cortex/RESULT.md`

Status: `READY_FOR_L3_REVIEW`
