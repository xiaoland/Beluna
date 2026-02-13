# L3a Plan - Comprehensive Implementation Checklist
- Task Name: `core-cortex-act-stem-refactor`
- Stage: `L3a` (strict execution checklist)
- Date: `2026-02-13`
- Status: `DRAFT_FOR_APPROVAL`

Follow this checklist in order without skipping gates.

## 1) Pre-Change Gate
1. confirm L3 package files are present.
2. snapshot `git status`.
3. keep unrelated workspace changes untouched.

## 2) Contract Cutover Gate
1. add `runtime_types.rs`.
2. remove admission module and exports.
3. replace admission imports/types in cortex/continuity/spine/body.
4. run `cargo check`.

Gate:
1. no compile-time references to `crate::admission`.

## 3) Stem Runtime Gate
1. add `ingress.rs` gate wrapper.
2. add `stem.rs` loop with:
   - control sense handling,
   - physical state compose,
   - cortex call,
   - cognition persistence,
   - inline serial act dispatch.
3. switch `main.rs` to `stem::run`.
4. run `cargo check`.

Gate:
1. runtime entry uses Stem; no Act queue remains.

## 4) Module Adapter Gate
1. adapt cortex outputs to `Act[]`.
2. adapt continuity APIs for cognition persistence and capability patches.
3. adapt spine to single-act dispatch request API.
4. adapt body std endpoint invocation contracts.
5. run targeted checks/tests.

Gate:
1. dispatch path compiles end-to-end.

## 5) Queue/Shutdown/Wire Gate
1. enforce bounded mpsc queue only.
2. implement shutdown sequence:
   - close ingress gate,
   - blocking sleep enqueue.
3. remove admission-feedback wire envelopes.
4. add patch/drop capability wire support.
5. update config/schema fields and defaults.

Gate:
1. shutdown behavior is deterministic and testable.

## 6) Test Gate
1. delete obsolete admission tests.
2. add/update stem tests.
3. run targeted suites:
   - `cargo test cortex:: -- --nocapture`
   - `cargo test continuity:: -- --nocapture`
   - `cargo test spine:: -- --nocapture`
   - `cargo test ledger:: -- --nocapture`
   - `cargo test stem:: -- --nocapture`
4. run full suite:
   - `cargo test -- --nocapture`

Gate:
1. all relevant tests green or pre-existing unrelated failures documented.

## 7) Documentation Gate
1. update overview/glossary/index docs.
2. remove admission references in active docs.
3. write final `docs/task/RESULT.md`.

Gate:
1. docs and code contracts match.

## 8) Finalization Gate
1. verify changed file set only.
2. commit with clear summary.
3. push branch and record PR-ready notes.

Status: `READY_FOR_EXECUTION`
