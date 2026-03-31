# L3-05 - Test Execution Plan

- Task Name: `mind-layer-mvp`
- Stage: `L3` detail: verification plan
- Date: 2026-02-09
- Status: `DRAFT_FOR_APPROVAL`

## 1) Test Layers

1. Unit tests (`tests/mind/*`)
- goal invariants and lifecycle transitions,
- preemption disposition rules and safe-point constraints,
- conflict resolver ownership and tie-break determinism,
- evolution trigger threshold behavior,
- deterministic facade loop ordering.

2. Contract conformance checks (via test naming and assertions)
- each `docs/contracts/mind/*` contract scenario has at least one corresponding `given_when_then` test.

3. Regression layer
- run full project test suite to ensure no impact to existing runtime/gateway paths.

## 2) Fixture Strategy

1. deterministic policy doubles
- fixed safe-point policy,
- fixed preemption/evaluator/evolution stubs when needed.

2. no-op adapter fixtures
- `NoopDelegationCoordinator`,
- `NoopMemoryPolicy`.

3. deterministic IDs
- use constant IDs/tokens in tests; avoid random UUID generation in assertions.

## 3) Command Sequence

1. `cargo fmt --check`
2. `cargo test`
3. focused runs if diagnosis needed:
- `cargo test mind::goal_manager`
- `cargo test mind::preemption`
- `cargo test mind::facade_loop`

## 4) Minimum Acceptance Gate

Implementation is accepted only if:

1. all existing tests remain green,
2. new Mind tests are green,
3. deterministic loop tests are green,
4. no socket/protocol integration side effects are introduced,
5. evolution remains proposal-only in code paths.

## 5) Known Test Limits (MVP)

1. no real helper-process integration tests,
2. no persistent memory store tests,
3. no runtime protocol tests for Mind exposure (explicitly out of scope).

## 6) Failure Triage Order

1. goal/invariant failures,
2. preemption/safe-point failures,
3. conflict/evolution policy failures,
4. facade loop ordering failures,
5. documentation/index linkage failures.

Status: `READY_FOR_L3_REVIEW`
