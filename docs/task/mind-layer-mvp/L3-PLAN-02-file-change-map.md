# L3-02 - File Change Map

- Task Name: `mind-layer-mvp`
- Stage: `L3` detail: file-level implementation map
- Date: 2026-02-09
- Status: `DRAFT_FOR_APPROVAL`

## 1) Files To Modify

1. `src/lib.rs`
- add `pub mod mind;`.

2. `docs/features/README.md`
- add Mind feature entry.

3. `docs/modules/README.md`
- add Mind module entry.

4. `docs/contracts/README.md`
- add Mind contracts index entry.

5. `docs/product/overview.md`
- add/update concise Mind MVP capability and limitation wording.

6. `docs/product/glossary.md`
- add missing Mind terms only if required by implemented types.

7. `AGENTS.md`
- refresh live capability/limitation lines if needed after implementation.

## 2) Files To Add (Mind Core)

1. `src/mind/mod.rs`
- module exports and public API surface.

2. `src/mind/error.rs`
- `MindErrorKind`, `MindError`, helper constructors.

3. `src/mind/types.rs`
- identifiers, commands, events, decisions, safe point, evaluation/conflict/evolution types.

4. `src/mind/state.rs`
- `MindState` and invariant-safe helpers.

5. `src/mind/goal_manager.rs`
- goal registration/activation/pause/cancel/merge operations.

6. `src/mind/preemption.rs`
- preemption context, decision types, default deterministic decider.

7. `src/mind/evaluator.rs`
- normative evaluator contract + deterministic default evaluator.

8. `src/mind/conflict.rs`
- conflict case model and deterministic resolver.

9. `src/mind/evolution.rs`
- proposal-only evolution decider and threshold logic.

10. `src/mind/ports.rs`
- `DelegationCoordinatorPort`, `MemoryPolicyPort`, directives.

11. `src/mind/facade.rs`
- deterministic loop implementation for `MindFacade::step`.

12. `src/mind/noop.rs`
- no-op delegation and memory policy adapters.

## 3) Files To Add (Tests)

1. `tests/mind/mod.rs`
- shared fixtures and helper constructors.

2. `tests/mind/goal_manager.rs`
- goal lifecycle and invariant tests.

3. `tests/mind/preemption.rs`
- disposition and safe-point constraint tests.

4. `tests/mind/evaluator.rs`
- normative judgment schema tests.

5. `tests/mind/conflict.rs`
- scoped conflict ownership and tie-break tests.

6. `tests/mind/evolution.rs`
- proposal-only trigger and threshold tests.

7. `tests/mind/facade_loop.rs`
- deterministic loop ordering and stability tests.

8. `tests/mind_bdt.rs`
- compile linkage entrypoint (mirror existing ai_gateway_bdt style).

## 4) Files To Add (Contracts)

1. `docs/contracts/mind/README.md`
2. `docs/contracts/mind/goal-management.md`
3. `docs/contracts/mind/preemption.md`
4. `docs/contracts/mind/evaluation.md`
5. `docs/contracts/mind/delegation-and-conflict.md`
6. `docs/contracts/mind/evolution-trigger.md`
7. `docs/contracts/mind/facade-loop.md`

## 5) Files To Add (Feature Docs)

1. `docs/features/mind/README.md`
2. `docs/features/mind/PRD.md`
3. `docs/features/mind/HLD.md`
4. `docs/features/mind/LLD.md`

## 6) Files To Add (Module Docs)

1. `docs/modules/mind/README.md`
2. `docs/modules/mind/purpose.md`
3. `docs/modules/mind/architecture.md`
4. `docs/modules/mind/execution-flow.md`
5. `docs/modules/mind/policies.md`

## 7) Files To Add (Task Artifact)

1. `docs/task/mind-layer-mvp/RESULT.md`
- implementation outcome, deviations, tests/evidence, known limitations.

Status: `READY_FOR_L3_REVIEW`
