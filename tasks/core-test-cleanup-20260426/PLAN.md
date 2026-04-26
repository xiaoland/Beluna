# Core Test Cleanup

> Last updated: 2026-04-26
> Status: execute packet
> Scope: hard cut stale Core tests before Agent Task testing design continues

## MVT Core

- Objective & Hypothesis: Remove stale and low-signal Core tests so the remaining verification surface stops preserving old API shape and future test work can center on Agent Task completion.
- Guardrails Touched: Runtime behavior must remain unchanged; cleanup must not weaken production boundary code, only remove obsolete test code and fixtures.
- Verification: `cargo check --manifest-path core/Cargo.toml`, `cargo test --manifest-path core/Cargo.toml --lib --no-run`, and `cargo test --manifest-path core/Cargo.toml --tests --no-run` complete after cleanup.

## Exploration Scaffold

- Perturbation: Core tests currently fail to compile because many test files reference old APIs and data models.
- Input Type: Intent.
- Active Mode or Transition Note: Execute after human approval for hard cut-off deletion.
- Governing Anchors:
  - `/Users/lanzhijiang/Development/Beluna/AGENTS.md`
  - `/Users/lanzhijiang/Development/Beluna/core/AGENTS.md`
  - `/Users/lanzhijiang/Development/Beluna/core/src/body/AGENTS.md`
  - `/Users/lanzhijiang/Development/Beluna/core/src/spine/AGENTS.md`
  - `/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/AGENTS.md`
  - `/Users/lanzhijiang/Development/Beluna/core/src/continuity/AGENTS.md`
  - `/Users/lanzhijiang/Development/Beluna/core/src/ledger/AGENTS.md`
  - `/Users/lanzhijiang/Development/Beluna/tasks/agent-task-testing-20260425/PLAN.md`
- Impact Hypothesis: A small empty-or-near-empty test surface is preferable to a broad stale test surface during the transition toward Agent Task testing.
- Temporary Assumptions:
  - Existing `core/tests` BDT files are historical scaffolding and can be deleted.
  - Inline tests can also be deleted in this cleanup round; precise mechanical guardrails can be reintroduced later in better locations.
  - New Agent Task tests are explicitly out of scope for this packet.
- Negotiation Triggers:
  - Production code behavior must change to make cleanup compile.
  - A local AGENTS rule requires a durable documentation decision before deletion can proceed.
- Promotion Candidates:
  - Test placement policy: inline tests for white-box micro-invariants, `tests/` for black-box public API and task-level integration.
  - AI Gateway AGENTS may need a later update because it still points tests toward `../../tests/ai_gateway/*`.

## Execution Plan

1. Delete all existing `core/tests` test files and fixtures.
2. Remove inline `#[cfg(test)]` modules and test-only module includes from `core/src`.
3. Run compile gates.
4. Record any residual failures that are production-code issues rather than test cleanup issues.

## Out Of Scope

- Agent Task runner implementation.
- AIMock integration.
- Durable documentation updates.
- Runtime behavior changes.

## Execution Notes

- Key findings:
  - Existing Core tests and fixtures were deleted as a hard cut.
  - Inline `#[cfg(test)]` modules were removed from Core source files.
  - No `#[test]`, `#[tokio::test]`, or `#[cfg(test)]` markers remain under `core/src` or `core/tests`.
- Decisions made:
  - Hard cut-off approved by human.
- Final outcome:
  - `cargo check --manifest-path core/Cargo.toml` passed.
  - `cargo test --manifest-path core/Cargo.toml --lib --no-run` passed.
  - `cargo test --manifest-path core/Cargo.toml --tests --no-run` passed.
  - `cargo test --manifest-path core/Cargo.toml` passed with zero remaining tests.
