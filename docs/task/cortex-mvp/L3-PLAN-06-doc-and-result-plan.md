# L3-06 - Doc And Result Plan
- Task Name: `cortex-mvp`
- Stage: `L3` detail: docs and closure
- Date: `2026-02-11`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Docs To Update
1. `/Users/lanzhijiang/Development/Beluna/docs/features/cortex/PRD.md`
- replace step-centric language with reactor semantics.

2. `/Users/lanzhijiang/Development/Beluna/docs/features/cortex/HLD.md`
- update boundary to `ReactionInput -> ReactionResult`.

3. `/Users/lanzhijiang/Development/Beluna/docs/features/cortex/LLD.md`
- add IR -> extractor/filler -> clamp -> one-repair pipeline.

4. `/Users/lanzhijiang/Development/Beluna/docs/contracts/cortex/README.md`
- add `attempt_id` + `based_on` requirement and feedback correlation rule.

5. `/Users/lanzhijiang/Development/Beluna/docs/overview.md`
- reflect `Sense + EnvSnapshot stream` and always-on progression wording.

6. `/Users/lanzhijiang/Development/Beluna/core/AGENTS.md`
- refresh current-state capabilities and known limitations.

## 2) Required Consistency Pass
1. remove stale mentions of `CortexFacade::step` as canonical API.
2. ensure no doc text implies cortex durable goal/commitment persistence.
3. ensure business output purity language stays explicit.

## 3) Final Result Document
Create:
1. `/Users/lanzhijiang/Development/Beluna/docs/task/cortex-mvp/RESULT.md`

Required sections:
1. implemented scope summary,
2. key architecture changes,
3. contract-level behavior guarantees,
4. test execution summary,
5. known limitations/follow-ups.

## 4) Completion Checklist
1. code changes merged and compile/tests pass.
2. docs and contracts aligned with implementation.
3. `RESULT.md` written and verified.

Status: `READY_FOR_L3_REVIEW`
