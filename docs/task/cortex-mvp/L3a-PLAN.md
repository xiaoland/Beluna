# L3a Plan - Comprehensive Implementation Checklist
- Task Name: `cortex-mvp`
- Stage: `L3a` (strict execution checklist)
- Date: `2026-02-11`
- Status: `DRAFT_FOR_APPROVAL`

Follow this checklist in order without skipping gates.

## 1) Pre-Change Gate
1. confirm L3 files approved.
2. snapshot current test baseline (`cargo test` optional pre-run if time allows).
3. avoid modifying unrelated files.

## 2) Contract Cutover
1. implement reactor contract types in `cortex/types.rs`.
2. update `cortex/mod.rs` exports.
3. remove step-centric canonical exports.
4. compile check.

Gate:
1. no unresolved references to old canonical step types in cortex paths.

## 3) Clamp + Port Foundation
1. implement async cognition ports.
2. implement deterministic clamp module.
3. extend `IntentAttempt` with `based_on`.
4. compile check.

Gate:
1. clamp unit tests for schema/catalog/based_on rules pass.

## 4) Reactor Engine
1. implement `react_once` and `run` loop.
2. enforce one-primary/N-subcall/one-repair policies.
3. implement noop fallback path.
4. compile check.

Gate:
1. reactor boundedness tests pass.

## 5) AI Gateway Adapters
1. implement primary/extractor/filler ai-gateway adapters.
2. map adapter failures to cycle-local noop outcomes.
3. add mock adapters for tests.
4. compile check.

Gate:
1. adapter tests pass with mocks and without network IO.

## 6) Runtime/Protocol Wiring
1. update protocol with ingress event types.
2. add server ingress assembler.
3. spawn and lifecycle-manage reactor task with bounded channels.
4. preserve exit flow behavior.
5. compile check.

Gate:
1. protocol tests and runtime smoke tests pass.

## 7) Integration + Regression Tests
1. update cortex integration flow for `attempt_id` feedback correlation.
2. update admission tests for `based_on` field.
3. run targeted test commands.
4. run full `cargo test`.

Gate:
1. all tests green.

## 8) Documentation Closure
1. update feature/contract/overview/AGENTS docs.
2. write `docs/task/cortex-mvp/RESULT.md`.
3. verify docs match implemented behavior.

## 9) Final Validation
1. re-run full tests if docs touched only code-adjacent artifacts are unaffected.
2. verify `git status` for intended file set only.
3. prepare concise implementation summary with file references.

Status: `READY_FOR_EXECUTION_AFTER_APPROVAL`
