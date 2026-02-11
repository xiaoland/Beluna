# L3-06 - Documentation And Result Plan
- Task Name: `refactor-mind-cortex-non-cortex`
- Stage: `L3` detail: docs migration and result capture
- Date: `2026-02-10`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Documentation Objectives
1. make `cortex/non-cortex/spine` canonical in docs.
2. eliminate `mind` as authoritative concept in code-facing docs.
3. capture mechanical invariants and contracts clearly.

## 2) Docs To Add

### 2.1 Features
1. `/Users/lanzhijiang/Development/Beluna/docs/features/cortex/README.md`
2. `/Users/lanzhijiang/Development/Beluna/docs/features/cortex/PRD.md`
3. `/Users/lanzhijiang/Development/Beluna/docs/features/cortex/HLD.md`
4. `/Users/lanzhijiang/Development/Beluna/docs/features/cortex/LLD.md`
5. `/Users/lanzhijiang/Development/Beluna/docs/features/non-cortex/*`
6. `/Users/lanzhijiang/Development/Beluna/docs/features/spine/*`

### 2.2 Modules
1. `/Users/lanzhijiang/Development/Beluna/docs/modules/cortex/*`
2. `/Users/lanzhijiang/Development/Beluna/docs/modules/non-cortex/*`
3. `/Users/lanzhijiang/Development/Beluna/docs/modules/spine/*`

### 2.3 Contracts
1. `/Users/lanzhijiang/Development/Beluna/docs/contracts/cortex/*`
2. `/Users/lanzhijiang/Development/Beluna/docs/contracts/non-cortex/*`
3. `/Users/lanzhijiang/Development/Beluna/docs/contracts/spine/*`

## 3) Docs To Modify
1. `/Users/lanzhijiang/Development/Beluna/docs/features/README.md`
2. `/Users/lanzhijiang/Development/Beluna/docs/modules/README.md`
3. `/Users/lanzhijiang/Development/Beluna/docs/contracts/README.md`
4. `/Users/lanzhijiang/Development/Beluna/docs/product/overview.md`
5. `/Users/lanzhijiang/Development/Beluna/docs/product/glossary.md`

## 4) Legacy Mind Docs Handling
1. `docs/features/mind/*`, `docs/modules/mind/*`, `docs/contracts/mind/*`:
- either remove, or mark explicitly as superseded with pointers.
2. no mixed-authority wording after migration.

## 5) Required Terminology Updates
1. "Goal != Commitment" explicitly documented in cortex docs.
2. "Constraints are affordance/economics, not narration" in non-cortex docs.
3. "Spine executes admitted actions only" in spine contracts.
4. "Versioned determinism tuple" in non-cortex policy/contract docs.

## 6) Final Result File
Create:
1. `/Users/lanzhijiang/Development/Beluna/docs/task/refactor-mind-cortex-non-cortex/RESULT.md`

Structure:
1. objective and scope delivered.
2. architecture before/after snapshot.
3. key invariants implemented.
4. file changes summary.
5. test results summary.
6. known limitations and next steps.

## 7) Completion Checklist
1. all new docs linked from indexes.
2. no stale references to removed `mind` APIs in core docs.
3. result file includes evidence (test commands and outcomes).
4. terminology is consistent with approved L1/L2 decisions.

Status: `READY_FOR_L3_REVIEW`
