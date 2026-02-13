# L3 Plan 06 - Documentation And Result Plan
- Task Name: `core-cortex-act-stem-refactor`
- Stage: `L3`
- Focus: documentation execution and result artifact closure
- Status: `DRAFT_FOR_APPROVAL`

## 1) Documentation Execution Order
1. update runtime overview first.
2. update glossary and contract indexes.
3. remove admission doc references.
4. update module/feature pages impacted by runtime cutover.
5. write final `docs/task/RESULT.md`.

## 2) Required Doc Updates
1. `/Users/lanzhijiang/Development/Beluna/docs/overview.md`
2. `/Users/lanzhijiang/Development/Beluna/docs/glossary.md`
3. `/Users/lanzhijiang/Development/Beluna/docs/features/README.md`
4. `/Users/lanzhijiang/Development/Beluna/docs/modules/README.md`
5. `/Users/lanzhijiang/Development/Beluna/docs/contracts/README.md`
6. remove/update admission package docs under `docs/features/admission`, `docs/modules/admission`, `docs/contracts/admission`.

## 3) Terminology Migration Rules
1. replace `IntentAttempt` with `Act` in active docs.
2. remove `AdmittedAction` as a first-class boundary term in active flow docs.
3. remove Admission boundary as runtime stage.
4. add `Stem` as canonical orchestrator term.
5. add control senses:
   - `sleep`,
   - `new_capabilities`,
   - `drop_capabilities`.

## 4) `RESULT.md` Structure
Final `/Users/lanzhijiang/Development/Beluna/docs/task/RESULT.md` should include:
1. scope summary and locked decisions,
2. delivered architecture delta,
3. file-level change summary,
4. test evidence (targeted + full),
5. known limitations/deferred follow-ups,
6. migration impact notes.

## 5) Consistency Checklist
1. no doc index points to removed admission contracts.
2. overview flow diagram matches implemented runtime.
3. glossary terms align with code-level type names.
4. AGENTS/current-state summaries are aligned where applicable.

Status: `READY_FOR_EXECUTION`
