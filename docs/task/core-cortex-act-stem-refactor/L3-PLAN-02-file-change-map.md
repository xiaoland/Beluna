# L3 Plan 02 - File Change Map
- Task Name: `core-cortex-act-stem-refactor`
- Stage: `L3`
- Focus: executable file-level changes
- Status: `DRAFT_FOR_APPROVAL`

## 1) Core Source Changes

### 1.1 Add
1. `/Users/lanzhijiang/Development/Beluna/core/src/runtime_types.rs`
2. `/Users/lanzhijiang/Development/Beluna/core/src/ingress.rs`
3. `/Users/lanzhijiang/Development/Beluna/core/src/stem.rs`

### 1.2 Delete
1. `/Users/lanzhijiang/Development/Beluna/core/src/admission/AGENTS.md`
2. `/Users/lanzhijiang/Development/Beluna/core/src/admission/affordance.rs`
3. `/Users/lanzhijiang/Development/Beluna/core/src/admission/mod.rs`
4. `/Users/lanzhijiang/Development/Beluna/core/src/admission/resolver.rs`
5. `/Users/lanzhijiang/Development/Beluna/core/src/admission/types.rs`

### 1.3 Modify
1. `/Users/lanzhijiang/Development/Beluna/core/src/lib.rs`
2. `/Users/lanzhijiang/Development/Beluna/core/src/main.rs`
3. `/Users/lanzhijiang/Development/Beluna/core/src/brainstem.rs` (remove legacy loop or convert to compatibility shim)
4. `/Users/lanzhijiang/Development/Beluna/core/src/config.rs`
5. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/mod.rs`
6. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/types.rs`
7. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/ports.rs`
8. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/pipeline.rs`
9. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/clamp.rs`
10. `/Users/lanzhijiang/Development/Beluna/core/src/continuity/mod.rs`
11. `/Users/lanzhijiang/Development/Beluna/core/src/continuity/types.rs`
12. `/Users/lanzhijiang/Development/Beluna/core/src/continuity/state.rs`
13. `/Users/lanzhijiang/Development/Beluna/core/src/continuity/engine.rs`
14. `/Users/lanzhijiang/Development/Beluna/core/src/continuity/ports.rs`
15. `/Users/lanzhijiang/Development/Beluna/core/src/continuity/noop.rs`
16. `/Users/lanzhijiang/Development/Beluna/core/src/continuity/invariants.rs`
17. `/Users/lanzhijiang/Development/Beluna/core/src/ledger/ledger.rs`
18. `/Users/lanzhijiang/Development/Beluna/core/src/spine/mod.rs`
19. `/Users/lanzhijiang/Development/Beluna/core/src/spine/types.rs`
20. `/Users/lanzhijiang/Development/Beluna/core/src/spine/ports.rs`
21. `/Users/lanzhijiang/Development/Beluna/core/src/spine/router.rs`
22. `/Users/lanzhijiang/Development/Beluna/core/src/spine/noop.rs`
23. `/Users/lanzhijiang/Development/Beluna/core/src/spine/adapters/wire.rs`
24. `/Users/lanzhijiang/Development/Beluna/core/src/spine/adapters/unix_socket.rs`
25. `/Users/lanzhijiang/Development/Beluna/core/src/spine/adapters/catalog_bridge.rs`
26. `/Users/lanzhijiang/Development/Beluna/core/src/body/mod.rs`
27. `/Users/lanzhijiang/Development/Beluna/core/src/body/shell.rs`
28. `/Users/lanzhijiang/Development/Beluna/core/src/body/web.rs`
29. `/Users/lanzhijiang/Development/Beluna/core/beluna.schema.json`

## 2) Test Changes

### 2.1 Add
1. `/Users/lanzhijiang/Development/Beluna/core/tests/stem_bdt.rs`
2. `/Users/lanzhijiang/Development/Beluna/core/tests/stem/mod.rs`
3. `/Users/lanzhijiang/Development/Beluna/core/tests/stem/loop.rs`
4. `/Users/lanzhijiang/Development/Beluna/core/tests/stem/shutdown.rs`
5. `/Users/lanzhijiang/Development/Beluna/core/tests/stem/capability_patch.rs`
6. `/Users/lanzhijiang/Development/Beluna/core/tests/stem/dispatch_pipeline.rs`

### 2.2 Delete
1. `/Users/lanzhijiang/Development/Beluna/core/tests/admission_bdt.rs`
2. `/Users/lanzhijiang/Development/Beluna/core/tests/admission/mod.rs`
3. `/Users/lanzhijiang/Development/Beluna/core/tests/admission/admission.rs`

### 2.3 Modify
1. `/Users/lanzhijiang/Development/Beluna/core/tests/cortex/mod.rs`
2. `/Users/lanzhijiang/Development/Beluna/core/tests/cortex/reactor.rs`
3. `/Users/lanzhijiang/Development/Beluna/core/tests/cortex/clamp.rs`
4. `/Users/lanzhijiang/Development/Beluna/core/tests/continuity/mod.rs`
5. `/Users/lanzhijiang/Development/Beluna/core/tests/continuity/debits.rs`
6. `/Users/lanzhijiang/Development/Beluna/core/tests/spine/mod.rs`
7. `/Users/lanzhijiang/Development/Beluna/core/tests/spine/contracts.rs`
8. `/Users/lanzhijiang/Development/Beluna/core/tests/cortex_continuity_flow.rs`
9. `/Users/lanzhijiang/Development/Beluna/core/tests/ledger/ledger.rs`

## 3) Documentation Changes
1. `/Users/lanzhijiang/Development/Beluna/docs/overview.md`
2. `/Users/lanzhijiang/Development/Beluna/docs/glossary.md`
3. `/Users/lanzhijiang/Development/Beluna/docs/features/README.md`
4. `/Users/lanzhijiang/Development/Beluna/docs/modules/README.md`
5. `/Users/lanzhijiang/Development/Beluna/docs/contracts/README.md`
6. admission package docs removal/update under:
   - `/Users/lanzhijiang/Development/Beluna/docs/features/admission/*`
   - `/Users/lanzhijiang/Development/Beluna/docs/modules/admission/*`
   - `/Users/lanzhijiang/Development/Beluna/docs/contracts/admission/*`
7. final result doc:
   - `/Users/lanzhijiang/Development/Beluna/docs/task/RESULT.md`

## 4) Notes
1. keep unrelated modified files untouched (`TODO.md`, `docs/task/README.md` unless explicitly requested).
2. avoid non-target behavior changes outside refactor scope.

Status: `READY_FOR_EXECUTION`
