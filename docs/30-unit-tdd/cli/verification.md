# CLI Verification

## Verification Scope

This file defines CLI-local verification contracts and expected evidence homes.

It does not redefine cross-unit contracts, system authority ownership, or decomposition policy.

## Local Verification Contracts

1. Registration contract realization:
- CLI registration payloads remain consistent with endpoint identity/capability expectations.
- Evidence homes: `cli/src/main.rs`, protocol-focused tests when registration shape changes.

2. Protocol framing and translation contract:
- NDJSON framing and field mapping remain deterministic for stdin sense emission and act rendering.
- Evidence homes: `cli/src/main.rs`, integration checks against core protocol behavior.

3. Terminal outcome handling contract:
- Non-ack outcomes and transport failures remain explicit to the operator.
- Evidence homes: `cli/src/main.rs`, failure-path tests for disconnect and invalid payload handling.

## Expected Guardrails

1. Wire-shape changes require synchronized updates to Product TDD contract docs and affected unit interface docs.
2. Error-path behavior must be verified alongside success paths for any protocol handling change.
3. CLI must not introduce hidden fallback logic that obscures contract violations.

## Escalation Rules

Escalate to Product TDD when a change affects:

1. Endpoint protocol, dispatch outcome, or identity contracts (`docs/20-product-tdd/cross-unit-contracts.md`).
2. Cross-unit authority ownership (`docs/20-product-tdd/system-state-and-authority.md`).
3. Unit split/merge or container mapping decisions (`docs/20-product-tdd/unit-boundary-rules.md`, `docs/20-product-tdd/unit-to-container-mapping.md`).
