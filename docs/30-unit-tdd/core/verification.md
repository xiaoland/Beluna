# Core Verification

## Verification Scope

This file defines core-local verification contracts and evidence expectations.

It does not redefine cross-unit contracts, decomposition policy, or system authority ownership.

## Local Verification Contracts

1. Config boundary contract:
- Typed config parsing/validation and schema compatibility remain explicit and deterministic.
- Evidence homes: `core/src/config/*`, `core/beluna.schema.json`, config boundary tests.

2. Continuity persistence contract:
- Continuity state persistence/restore remains deterministic and guardrailed under restart and error paths.
- Evidence homes: `core/src/continuity/*`, `core/tests/continuity/*`.

3. Dispatch terminal-outcome contract:
- Core dispatch produces explicit terminal outcomes and correlated failure signaling on reject/lost paths.
- Evidence homes: `core/src/spine/*`, `core/tests/spine/*`.

4. Runtime shutdown/recovery contract:
- Ingress gating, task cancellation, and bounded drain behavior remain consistent with runtime constraints.
- Evidence homes: `core/src/main.rs`, runtime integration tests, `docs/40-deployment/rollout-and-recovery.md`.

5. OTLP log inspection contract:
- Core emits a subsystem-owned OTLP family catalog that satisfies `docs/20-product-tdd/observability-contract.md`.
- Moira's consumed inspection surfaces remain reconstructable without treating free-form log text as the primary contract.
- Evidence homes: observability export code, OTLP event-shape tests, `docs/30-unit-tdd/core/observability.md`, and integration checks against Moira-facing contract fixtures.

## Expected Guardrails

1. Changes to boundary types, schemas, or terminal outcome semantics require matching updates to tests/contracts.
2. New stateful behavior must include failure-path verification, not only happy-path checks.
3. Any cross-unit contract implication must trigger Product TDD updates before unit-level acceptance.
4. OTLP family changes must update the family spec table and minimum fixture set in `docs/30-unit-tdd/core/observability.md`.

## Escalation Rules

Escalate to Product TDD when a change affects:

1. Endpoint protocol or identity semantics (`docs/20-product-tdd/cross-unit-contracts.md`).
2. Cross-unit authority ownership (`docs/20-product-tdd/system-state-and-authority.md`).
3. Unit boundary/decomposition decisions (`docs/20-product-tdd/unit-boundary-rules.md`).
