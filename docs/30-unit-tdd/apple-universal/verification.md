# Apple Universal Verification

## Verification Scope

This file defines Apple Universal local verification contracts and evidence expectations.

It does not redefine cross-unit contracts, system authority ownership, or decomposition policy.

## Local Verification Contracts

1. Connection lifecycle contract:
- Manual connect/disconnect/retry behavior and socket-path apply-on-reconnect remain explicit and deterministic.
- Evidence homes: `apple-universal/BelunaApp/App/*` lifecycle logic and focused lifecycle tests.

2. Protocol compatibility contract:
- Request/response encoding/decoding and correlation metadata handling remain typed and explicit.
- Evidence homes: `apple-universal/BelunaApp/BodyEndpoint/*`, protocol decode/encode tests.

3. Local history contract:
- Sense/act local persistence and bounded in-memory buffering behavior remain stable across relaunch and pagination flows.
- Evidence homes: `apple-universal/BelunaApp/App/LocalSenseActHistoryStore.swift`, `ChatViewModel` tests when behavior changes.

4. UI responsiveness contract:
- Network/protocol operations remain off main thread and do not block interaction surfaces.
- Evidence homes: runtime behavior checks and targeted unit/UI tests for reconnect and large-history scenarios.

## Expected Guardrails

1. Protocol shape changes require synchronized updates to Product TDD contract docs and endpoint unit docs.
2. Reconnect and pagination regressions must be validated in failure-oriented scenarios, not only nominal flows.
3. Local persistence format/logic changes must include restore compatibility checks.

## Escalation Rules

Escalate to Product TDD when a change affects:

1. Cross-unit protocol or identity semantics (`docs/20-product-tdd/cross-unit-contracts.md`).
2. Cross-unit authority ownership (`docs/20-product-tdd/system-state-and-authority.md`).
3. Unit boundary/container mapping decisions (`docs/20-product-tdd/unit-boundary-rules.md`, `docs/20-product-tdd/unit-to-container-mapping.md`).
