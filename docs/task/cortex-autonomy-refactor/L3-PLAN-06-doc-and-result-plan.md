# L3-06 Doc and Result Plan
- Task: `cortex-autonomy-refactor`
- Stage: `L3`

## 1) Documentation Update Sequence
1. update contracts first:
- `docs/contracts/cortex/README.md`
- `docs/contracts/stem/README.md`
- `docs/contracts/continuity/README.md`

2. update feature docs:
- `docs/features/cortex/*`
- `docs/features/stem/*`
- `docs/features/continuity/*`

3. update module docs:
- `docs/modules/cortex/README.md`
- `docs/modules/stem/README.md`
- `docs/modules/continuity/README.md`

4. update global docs:
- `docs/overview.md`
- `docs/glossary.md`

5. update agent docs:
- `core/AGENTS.md`
- `core/src/cortex/AGENTS.md`

## 2) Documentation Consistency Checks
1. no lingering `goal-stack` terminology in active docs.
2. no lingering `Sense::Sleep` runtime-control contract.
3. dispatch chain described as per-act middleware (`Continuity on_act -> Spine on_act`).
4. cortex boundary described as `acts + new_cognition_state`.
5. communication model explicitly names afferent/efferent producers and consumers.

## 3) Result Document Template
Target file:
- `docs/task/cortex-autonomy-refactor/RESULT.md`

Sections:
1. Date and status.
2. Implemented architecture changes.
3. File changes summary.
4. Build/test execution log.
5. Known limitations or follow-up items.

## 4) Exit Condition
Task closes only when:
1. code compiles,
2. docs reflect shipped behavior,
3. result document is complete.

