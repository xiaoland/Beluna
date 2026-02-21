# L2 Plan - Cortex Autonomy Refactor (Low-Level Design)
- Task Name: `cortex-autonomy-refactor`
- Stage: `L2` (low-level design)
- Date: `2026-02-21`
- Status: `DRAFT_FOR_APPROVAL`
- Input: approved `L1`

This L2 package is split into focused files so interfaces, data structures, algorithms, and migration sequence can be reviewed independently.

## L2 File Index
1. `/Users/lanzhijiang/Development/Beluna/docs/task/cortex-autonomy-refactor/L2-PLAN-01-domain-types-and-ir-contracts.md`
- Cortex-owned cognition data types (`goal-tree`, `l1-memory`)
- IR schema and patch algebra

2. `/Users/lanzhijiang/Development/Beluna/docs/task/cortex-autonomy-refactor/L2-PLAN-02-stem-scheduler-and-dispatch.md`
- Tick/sleep/hibernate scheduler state machine
- New per-act middleware path `Continuity on_act -> Spine on_act`

3. `/Users/lanzhijiang/Development/Beluna/docs/task/cortex-autonomy-refactor/L2-PLAN-03-continuity-storage-and-guardrails.md`
- JSON persistence layout and atomic IO algorithm
- Guardrail rules for root partition, goal-tree patch, and l1-memory patch

4. `/Users/lanzhijiang/Development/Beluna/docs/task/cortex-autonomy-refactor/L2-PLAN-04-cortex-prompt-and-helper-module.md`
- Single prompt module design (Primary + Helpers)
- Cortex pipeline changes for autonomous tick with possibly empty senses

5. `/Users/lanzhijiang/Development/Beluna/docs/task/cortex-autonomy-refactor/L2-PLAN-05-file-change-map-and-build-plan.md`
- File-by-file add/modify/delete map
- Build-only verification plan and doc sync checklist

## L2 Completion Gate
L2 is complete when:
1. data contracts are explicit and serializable,
2. scheduler and dispatch algorithms are unambiguous,
3. continuity persistence + guardrails are deterministic,
4. prompt/module ownership is fixed,
5. implementation can start directly with no architecture redesign.

Status: `READY_FOR_REVIEW`
