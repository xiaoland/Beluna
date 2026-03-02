# L3 Plan 06 - State Ownership and Continuity Refactor
- Task: `cortex-loop-architecture`
- Micro-task: `06-state-ownership-and-continuity-refactor`
- Stage: `L3`
- Date: `2026-03-02`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Objective
Implement state-boundary hardening by removing L1 memory paths, making Stem the canonical shared physical-state owner, and narrowing Continuity to cognition persistence + act gating.

## 2) Execution Steps
### Step 1 - Physical State Canonical Store Refactor (Stem-owned)
1. Refactor `StemPhysicalStateStore` to hold canonical `Arc<RwLock<PhysicalState>>`.
2. Keep `StemControlPort` as the only write surface:
- descriptor patch/drop mutate `capabilities`.
- proprioception patch/drop mutate `proprioception`.
3. Add read-handle/snapshot API for Cortex runtime:
- snapshot clones full physical state under short read lock.
- sets `cycle_id` on cloned value for current cycle.

### Step 2 - Main/Cortex Runtime Wiring
1. Update `main.rs` composition to pass the shared physical-state read handle to Cortex deps.
2. Update `PhysicalStateReadPort` usage in `cortex/runtime.rs` to consume the new shared-state snapshot path.
3. Preserve invariant: refresh physical snapshot (including proprioception) before each cycle dispatch to Primary.

### Step 3 - Hard Remove L1 Memory from Cognition Model
1. Remove `L1Memory` alias and `CognitionState.l1_memory`.
2. Update `CognitionState::default`.
3. Remove L1 branches from cognition patch application (`apply_cognition_patches` and overflow bookkeeping).

### Step 4 - Remove L1 Prompt/IR/Helper Paths
1. Remove focal-awareness input/output tags and parsing from `cortex/ir.rs`.
2. Remove focal-awareness prompt requirements from `cortex/prompts.rs`.
3. Remove helper modules:
- `l1_memory_input_helper`
- `l1_memory_flush_output_helper`
4. Remove helper wiring and organ route entries from `helpers/mod.rs`.
5. Update `primary.rs` pipeline:
- stop building focal-awareness section.
- stop running l1 flush helper stage.
- stop logging l1-flush output fields.

### Step 5 - Narrow Continuity Scope
1. Remove descriptor overlay state and APIs from `continuity/state.rs` and `continuity/engine.rs`.
2. Keep:
- cognition state persistence/load
- cognition validation
- `on_act` gate.
3. Update persistence contract/version according to hard-cut policy.

### Step 6 - Goal-Forest Deterministic Helper Access
1. Keep goal-forest helper as regular Cortex dependency wiring (no singleton/shared runtime accessor).
2. Ensure `to_input_ir_section` remains deterministic and LLM-free.
3. Keep patch sub-agent path separate from deterministic rendering path.

### Step 7 - Config/Schema and Hook Cleanup
1. Remove `max_l1_memory_entries` from:
- `cortex/types.rs`
- `config.rs`
- `beluna.schema.json`.
2. Remove `helper_routes.l1_memory_flush_helper` from config and schema.
3. Update `cortex/testing.rs` hook contracts and compile-time callsites to drop L1 helper hooks.

### Step 8 - Result Documentation Update
1. Append micro-task 06 implementation result in `docs/task/RESULT.md` with ownership and hard-cut notes.

## 3) File-Level Change Map
1. `core/src/stem/runtime.rs`
2. `core/src/main.rs`
3. `core/src/cortex/runtime.rs`
4. `core/src/cortex/cognition.rs`
5. `core/src/cortex/cognition_patch.rs`
6. `core/src/cortex/helpers/mod.rs`
7. `core/src/cortex/helpers/l1_memory_input_helper.rs` (remove)
8. `core/src/cortex/helpers/l1_memory_flush_output_helper.rs` (remove)
9. `core/src/cortex/primary.rs`
10. `core/src/cortex/ir.rs`
11. `core/src/cortex/prompts.rs`
12. `core/src/cortex/testing.rs`
13. `core/src/cortex/types.rs`
14. `core/src/config.rs`
15. `core/beluna.schema.json`
16. `core/src/continuity/engine.rs`
17. `core/src/continuity/state.rs`
18. `core/src/continuity/persistence.rs`
19. `docs/task/RESULT.md`

## 4) Verification Gates
### Gate A - L1 Path Removed
```bash
rg -n "l1_memory|focal-awareness|new-focal-awareness|l1_memory_flush_helper" core/src
```
Expected:
1. no active L1-memory runtime path under Cortex/Continuity contracts.

### Gate B - Physical State Shared Ownership
```bash
rg -n "Arc<RwLock<PhysicalState>>|snapshot_for_cycle|StemControlPort|PhysicalStateReadPort" core/src/stem core/src/cortex core/src/main.rs
```
Expected:
1. Stem owns canonical shared physical state.
2. Cortex reads via snapshot path only.

### Gate C - Continuity Scope Narrowing
```bash
rg -n "apply_neural_signal_descriptor_patch|apply_neural_signal_descriptor_drop|neural_signal_descriptor_snapshot|tombstoned_routes" core/src/continuity
```
Expected:
1. descriptor overlay state/APIs removed from continuity module.

### Gate D - Goal-Forest Deterministic Render
```bash
rg -n "goal_forest.*to_input_ir|run_organ\\(|GoalForestHelper" core/src/cortex/helpers/goal_forest_helper.rs core/src/cortex/helpers/mod.rs
```
Expected:
1. deterministic goal-forest input rendering path remains LLM-free.
2. helper remains DI-wired without singleton access pattern.

### Gate E - Build
Per workspace rule:
```bash
cd core && cargo build
cd ../cli && cargo build
```

## 5) Completion Criteria (06)
1. Stem is canonical owner/writer of shared physical state lock.
2. Cortex refreshes proprioception through per-cycle physical snapshot before Primary dispatch.
3. `l1_memory` is fully removed from cognition model and Cortex runtime contracts.
4. Continuity no longer owns descriptor overlay state.
5. Goal-forest deterministic render path is DI-wired and LLM-free.
6. Core and CLI build successfully.

Status: `READY_FOR_REVIEW`
