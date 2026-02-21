# L3-01 Workstreams and Sequence
- Task: `cortex-autonomy-refactor`
- Stage: `L3`

## 1) Execution Order

### WS0: Contract Freeze and Scaffolding
Scope:
1. add cortex cognition types and patch op types.
2. align `Sense` enum with `Hibernate` and remove sleep sense path.
3. freeze shared dispatch decision types.

Exit gate:
1. all new type definitions compile.
2. no runtime behavior changes yet.

### WS1: Cortex Pipeline Refactor
Scope:
1. add unified prompt module (`prompts.rs`).
2. migrate IR tags (`goal-tree`, `l1-memory`, new output patch tags).
3. add `goal_tree_helper` with cache (user partition only).
4. keep l1-memory passthrough.
5. remove wrapper output schemas and parse arrays directly.
6. patch-apply inside Cortex to produce `new_cognition_state`.

Exit gate:
1. cortex module compiles.
2. cortex unit tests updated for new contracts.

### WS2: Continuity Storage + Guardrails
Scope:
1. implement direct JSON load/store with atomic write.
2. validate full `new_cognition_state` on persist.
3. root partition immutability checks against compile-time constants.
4. keep `on_act` middleware hook as no-op `Continue`.

Exit gate:
1. continuity compiles with new API.
2. persistence guardrail tests compile and pass (optional focus).

### WS3: Stem Scheduler + Sleep Act
Scope:
1. implement `Active` and `SleepingUntil` loop modes.
2. configure interval with missed tick skip.
3. implement built-in sleep act interception (`seconds`).
4. persist returned cognition state via continuity each cycle.
5. per-act middleware dispatch: stem control -> continuity.on_act -> spine.on_act.

Exit gate:
1. stem compiles.
2. loop/shutdown tests updated.

### WS4: Spine Middleware Entry and Sense Feedback
Scope:
1. add `spine.on_act` middleware entry returning `Continue|Break`.
2. keep routing/dispatch internals mechanical.
3. convert dispatch failures to senses via afferent sender.

Exit gate:
1. spine compiles.
2. dispatch behavior remains deterministic.

### WS5: Main/Config/Schema Wiring
Scope:
1. add loop tick config + continuity state path config.
2. update JSON schema.
3. main emits `Sense::Hibernate` on shutdown.
4. wire afferent sender ownership into continuity/spine as required.

Exit gate:
1. `cargo build` succeeds in `core/`.

### WS6: Documentation and AGENTS Updates
Scope:
1. update feature/module/contract docs for cortex/stem/continuity.
2. update `core/AGENTS.md` and `core/src/cortex/AGENTS.md`.
3. update overview/glossary.

Exit gate:
1. docs reflect actual implemented behavior.

### WS7: Result Finalization
Scope:
1. write `docs/task/cortex-autonomy-refactor/RESULT.md`.
2. record build/test commands executed and outcomes.

Exit gate:
1. result document complete and traceable to L3 checklist.

## 2) Hard Dependency Rules
1. WS1 depends on WS0.
2. WS2 depends on WS0 and partially WS1 (cognition state structs).
3. WS3 depends on WS1 and WS2.
4. WS4 depends on WS3 dispatch contract.
5. WS5 depends on WS3/WS4 APIs.
6. WS6 and WS7 occur after code stabilization.

## 3) Stop/Go Conditions
Stop and ask user only if:
1. conflict emerges between approved L2 contracts and compile-time reality that cannot be reconciled locally,
2. a required decision was not covered in approved L2/L3 docs.

Otherwise proceed end-to-end.

