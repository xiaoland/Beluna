# L3-02 - File Change Map
- Task Name: `cortex-mvp`
- Stage: `L3` detail: file-level execution map
- Date: `2026-02-11`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Files To Add - Cortex Reactor
1. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/reactor.rs`
2. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/pipeline.rs`
3. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/clamp.rs`
4. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/adapters/mod.rs`
5. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/adapters/ai_gateway.rs`

## 2) Files To Modify - Cortex Core
1. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/mod.rs`
- export reactor/pipeline/clamp/adapters.
- retire step-centric exports.

2. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/types.rs`
- add `ReactionInput`, `ReactionResult`, sense/context/catalog/limits types.
- add prose IR/draft types.
- preserve deterministic id derivation helpers or rehome as needed.

3. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/ports.rs`
- replace old decomposer-only port with async cognition ports + clamp/telemetry ports.

4. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/error.rs`
- add reactor/pipeline error kinds.

5. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/AGENTS.md`
- update invariants for reactor model.

## 3) Files To Retire Or Remove - Step-Centric Cortex
1. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/facade.rs`
2. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/commitment_manager.rs`
3. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/planner.rs`
4. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/state.rs`
5. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/noop.rs`

Note:
1. if a file is partially reusable, refactor in place instead of delete/recreate.
2. no compatibility wrapper for `step` is planned.

## 4) Files To Modify - Shared Contracts
1. `/Users/lanzhijiang/Development/Beluna/core/src/admission/types.rs`
- extend `IntentAttempt` with required `based_on: Vec<SenseId>`.

2. `/Users/lanzhijiang/Development/Beluna/core/src/protocol.rs`
- add ingress message contracts for `sense`, `env_snapshot`, `admission_feedback`, optional updates.

3. `/Users/lanzhijiang/Development/Beluna/core/src/server.rs`
- run reactor task.
- add ingress assembler and bounded channels.

4. `/Users/lanzhijiang/Development/Beluna/core/src/config.rs`
- add cortex config block.

5. `/Users/lanzhijiang/Development/Beluna/core/beluna.schema.json`
- add schema for cortex config.

6. `/Users/lanzhijiang/Development/Beluna/core/src/lib.rs`
- ensure exports remain coherent after cortex cutover.

## 5) Files To Add/Modify - Tests
1. `/Users/lanzhijiang/Development/Beluna/core/tests/cortex/reactor.rs` (new)
2. `/Users/lanzhijiang/Development/Beluna/core/tests/cortex/clamp.rs` (new)
3. `/Users/lanzhijiang/Development/Beluna/core/tests/cortex/ai_gateway_adapter.rs` (new)
4. `/Users/lanzhijiang/Development/Beluna/core/tests/cortex/mod.rs` (modify)
5. `/Users/lanzhijiang/Development/Beluna/core/tests/cortex_continuity_flow.rs` (modify for `based_on` and feedback correlation)
6. `/Users/lanzhijiang/Development/Beluna/core/tests/admission/admission.rs` (modify `IntentAttempt` construction)
7. `/Users/lanzhijiang/Development/Beluna/core/tests/continuity/debits.rs` (modify if feedback types change)

Potential removals:
1. `/Users/lanzhijiang/Development/Beluna/core/tests/cortex/planner.rs`
2. `/Users/lanzhijiang/Development/Beluna/core/tests/cortex/commitments.rs`

## 6) Files To Modify - Documentation
1. `/Users/lanzhijiang/Development/Beluna/docs/features/cortex/PRD.md`
2. `/Users/lanzhijiang/Development/Beluna/docs/features/cortex/HLD.md`
3. `/Users/lanzhijiang/Development/Beluna/docs/features/cortex/LLD.md`
4. `/Users/lanzhijiang/Development/Beluna/docs/contracts/cortex/README.md`
5. `/Users/lanzhijiang/Development/Beluna/docs/overview.md`
6. `/Users/lanzhijiang/Development/Beluna/core/AGENTS.md`

## 7) Task Output File
1. `/Users/lanzhijiang/Development/Beluna/docs/task/cortex-mvp/RESULT.md`

Status: `READY_FOR_L3_REVIEW`
