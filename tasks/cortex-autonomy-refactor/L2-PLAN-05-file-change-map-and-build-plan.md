# L2-05 File Change Map and Build Plan
- Task: `cortex-autonomy-refactor`
- Stage: `L2`

## 1) Core Code Change Map

### 1.1 Add Files
1. `core/src/cortex/cognition.rs`
- `GoalNode`, `GoalTree`, `L1Memory`, `CognitionState` (cortex-owned)
- patch op types

2. `core/src/cortex/prompts.rs`
- unified prompt builders for Primary + Helpers

3. `core/src/continuity/persistence.rs`
- direct JSON load/store helpers (no abstraction layer)

### 1.2 Modify Files
1. `core/src/types.rs`
- add `Sense::Hibernate`
- remove `Sense::Sleep`
- align shared cognition type exports with cortex-owned contracts

2. `core/src/cortex/mod.rs`
- export cognition and patch types
- wire prompts module

3. `core/src/cortex/types.rs`
- keep final `CortexOutput = acts + new_cognition_state`
- separate internal IR structs from boundary output

4. `core/src/cortex/ir.rs`
- InputIR tags: `senses`, `act-descriptor-catalog`, `goal-tree`, `l1-memory`
- OutputIR tags: `acts`, `goal-tree-patch`, `l1-memory-patch`

5. `core/src/cortex/helpers_input.rs`
- add `goal_tree_helper` flow (user partition only)
- add cache for goal-tree helper
- l1-memory passthrough section

6. `core/src/cortex/helpers_output.rs`
- replace goal-stack helpers with goal-tree/l1-memory patch helpers
- helper schemas return arrays directly (no `acts`/`patch` wrapper)

7. `core/src/cortex/runtime.rs`
- remove empty-sense rejection
- use unified prompts module
- run 3 input helpers and 3 output helpers
- apply patches inside cortex and emit full new cognition state

8. `core/src/cortex/AGENTS.md`
- update invariants and IR contracts

9. `core/src/continuity/state.rs`
- remove spine event tracking paths
- validate and persist full `new_cognition_state`
- implement `on_act` middleware hook (currently no-op continue)

10. `core/src/continuity/engine.rs`
- expose `persist_cognition_state` guardrail path and `on_act`

11. `core/src/continuity/types.rs`
- prune obsolete dispatch record types
- keep middleware dispatch context types

12. `core/src/continuity/mod.rs`
- export persistence and middleware API surface

13. `core/src/stem.rs`
- modeful tick scheduler (`Active` / `SleepingUntil`)
- sleep act interception with seconds payload
- per-act middleware chain: stem control -> continuity on_act -> spine on_act

14. `core/src/main.rs`
- signal handler emits `Sense::Hibernate`
- pass afferent sender to continuity and spine

15. `core/src/config.rs`
- add tick config fields
- add continuity `state_path`

16. `core/beluna.schema.json`
- schema updates for new loop/continuity config and helper routes

17. `core/src/spine/runtime.rs`
- add `on_act` middleware-style entry (`Continue|Break`)
- dispatch failures mapped to senses emitted via afferent sender

18. `core/AGENTS.md`
- update runtime model, dispatch path, communication model

### 1.3 Potential Deletions/Simplifications
1. old goal-stack structs and helper route configs
2. continuity dispatch record buffers and spine event ingestion code
3. stem ledger dispatch/settlement helpers

## 2) Docs Change Map
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

## 3) Test Impact Map
1. `core/tests/stem/*`
- tick/sleep/hibernate behavior
- sleep act interception with seconds
- per-act middleware dispatch

2. `core/tests/cortex/*`
- new IR tags
- helper array schemas
- final output `acts + new_cognition_state`
- goal-tree helper cache

3. `core/tests/continuity/*`
- full-state persist guardrails
- root partition immutability checks
- JSON load/store behavior

4. `core/tests/spine/*`
- on_act middleware behavior
- dispatch-failure->sense emission

## 4) Build and Verification Plan (Per Workspace Rule)
Build is the required gate:
1. `cd /Users/lanzhijiang/Development/Beluna/core`
2. `cargo build`

Optional focused checks:
1. `cargo test --test stem_bdt`
2. `cargo test --test cortex_bdt`

If optional tests are skipped, record in result doc.

## 5) Implementation Readiness Checklist
L3 can start when:
1. cognition contracts and naming are approved,
2. sleep/hibernate + middleware dispatch model is approved,
3. afferent/efferent communication model is approved,
4. prompt+helper ownership and helper cache plan are approved,
5. file map is approved.

