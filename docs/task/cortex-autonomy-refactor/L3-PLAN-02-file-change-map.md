# L3-02 File Change Map
- Task: `cortex-autonomy-refactor`
- Stage: `L3`

## 1) Add
1. `core/src/cortex/cognition.rs`
2. `core/src/cortex/prompts.rs`
3. `core/src/continuity/persistence.rs`
4. `docs/task/cortex-autonomy-refactor/RESULT.md`

## 2) Modify Core Runtime
1. `core/src/types.rs`
2. `core/src/cortex/mod.rs`
3. `core/src/cortex/types.rs`
4. `core/src/cortex/ir.rs`
5. `core/src/cortex/helpers_input.rs`
6. `core/src/cortex/helpers_output.rs`
7. `core/src/cortex/runtime.rs`
8. `core/src/cortex/AGENTS.md`
9. `core/src/continuity/mod.rs`
10. `core/src/continuity/types.rs`
11. `core/src/continuity/state.rs`
12. `core/src/continuity/engine.rs`
13. `core/src/stem.rs`
14. `core/src/main.rs`
15. `core/src/config.rs`
16. `core/src/spine/mod.rs`
17. `core/src/spine/runtime.rs`
18. `core/beluna.schema.json`
19. `core/AGENTS.md`

## 3) Modify Tests
1. `core/tests/stem/loop_control.rs`
2. `core/tests/stem/shutdown.rs`
3. `core/tests/stem/capability_patch.rs`
4. `core/tests/stem/dispatch_pipeline.rs`
5. `core/tests/cortex/pipeline.rs`
6. `core/tests/cortex/clamp.rs`
7. `core/tests/continuity/state.rs`
8. `core/tests/spine/dispatch.rs`

## 4) Update Docs
1. `docs/features/cortex/PRD.md`
2. `docs/features/cortex/HLD.md`
3. `docs/features/cortex/LLD.md`
4. `docs/features/stem/PRD.md`
5. `docs/features/stem/HLD.md`
6. `docs/features/stem/LLD.md`
7. `docs/features/continuity/PRD.md`
8. `docs/features/continuity/HLD.md`
9. `docs/features/continuity/LLD.md`
10. `docs/modules/cortex/README.md`
11. `docs/modules/stem/README.md`
12. `docs/modules/continuity/README.md`
13. `docs/contracts/cortex/README.md`
14. `docs/contracts/stem/README.md`
15. `docs/contracts/continuity/README.md`
16. `docs/overview.md`
17. `docs/glossary.md`

## 5) Remove/Simplify During Refactor
1. goal-stack specific helper and patch types.
2. continuity spine-event recording and dispatch record buffers.
3. stem ledger dispatch wiring and synthetic settlement mapping.
4. `Sense::Sleep` control path.

