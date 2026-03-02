# L2 Plan 06 - State Ownership and Continuity Refactor
- Task: `cortex-loop-architecture`
- Micro-task: `06-state-ownership-and-continuity-refactor`
- Stage: `L2`
- Date: `2026-03-02`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Goal and Scope
Goal:
1. Lock state ownership boundaries:
- physical state is Stem-owned shared state.
- cognition persistence is Cortex -> Continuity direct call.
2. Hard-remove `l1_memory` from cognition model and all Cortex contracts.
3. Narrow Continuity to cognition persistence + act gate responsibilities.
4. Make goal-forest input rendering deterministic with non-singleton DI lifecycle.

In scope:
1. `PhysicalState` sharing model (`Arc<RwLock<PhysicalState>>`) and read policy.
2. Cognition model simplification (`CognitionState` shape).
3. Cortex prompt/IR/helper contract cleanup for removed focal-awareness path.
4. Continuity state/API cleanup for ownership consistency.

Out of scope:
1. Efferent FIFO redesign (`07`).
2. Module-doc full refresh (`08`) beyond touched contract references.

## 2) Ownership Contract Freeze
Physical state:
1. Canonical physical state is held inside Stem as `Arc<RwLock<PhysicalState>>`.
2. Stem is the only writer via `StemControlPort` patch/drop operations.
3. Cortex uses read-only snapshot access and never acquires write lock.
4. Snapshot lock must be short-lived: clone under read lock, then release before any heavy/await work.

Cycle semantics:
1. `cycle_id` remains Cortex-runtime-owned sequencing.
2. `snapshot_for_cycle(cycle_id)` sets per-cycle `cycle_id` on cloned snapshot before Primary call.

Cognition state:
1. `CognitionState` becomes:
- `revision: u64`
- `goal_forest: GoalForest`
2. No `l1_memory` field, no focal-awareness section, no l1 flush step.

## 3) Cortex Contract Freeze (Post-L1 Removal)
Input IR:
1. Keep:
- `<somatic-senses>`
- `<proprioception>`
- `<somatic-act-descriptor-catalog>`
- `<goal-forest>`
2. Remove:
- `<focal-awareness>`

Output parsing:
1. Remove `<new-focal-awareness>` extraction path entirely.
2. Primary output is interpreted only for tool-call flow and any retained deterministic tags (none for L1).

Helper model:
1. Remove `l1_memory_input_helper` and `l1_memory_flush_output_helper`.
2. Remove `CognitionOrgan::L1MemoryFlush` route and associated helper route config.

## 4) Continuity Contract Freeze
Continuity responsibilities:
1. Persist and load `CognitionState`.
2. Validate cognition state invariants (goal forest topology, weight bounds, IDs).
3. Provide deterministic `on_act` gate decision.

Removed from Continuity:
1. Neural-signal descriptor overlay state and versioning.
2. Descriptor patch/drop APIs and snapshot APIs.

Persistence strategy:
1. Hard cut is allowed (no backward compatibility requirement).
2. Persistence format/version update is permitted to enforce new cognition-state contract.

## 5) Goal-Forest Helper Freeze
1. Goal-forest `to_input_ir_section` is deterministic pure rendering path.
2. No LLM transformation in `to_input_ir_section`.
3. Helper access remains regular dependency wiring inside Cortex helper graph (no singleton/shared runtime instance).
4. Patch conversion via sub-agent remains as a separate, explicit path (not part of `to_input_ir` rendering).

## 6) Config/Schema Freeze
1. Remove `max_l1_memory_entries` from `ReactionLimits`, config loader, and JSON schema.
2. Remove `helper_routes.l1_memory_flush_helper` from config and schema.
3. Keep other reaction limits unchanged.

## 7) File/Interface Freeze for L3
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

## 8) Risks and Constraints
1. Hard L1 removal can break many test fixtures/hook signatures.
Mitigation: stage type-signature updates first, then remove helper modules.
2. Shared physical-state lock may introduce lock contention.
Mitigation: enforce clone-and-release lock policy in Cortex runtime.
3. Continuity persistence hard cut can reject old state files.
Mitigation: explicit migration policy note; acceptable by current non-compat requirement.

## 9) L2 Exit Criteria (06)
1. Ownership boundaries for physical and cognition state are unambiguous.
2. `l1_memory` removal scope is complete and file-level.
3. Continuity scope is narrowed and descriptor overlay removal is explicit.
4. Deterministic non-singleton goal-forest rendering contract is locked.

Status: `READY_FOR_REVIEW`
