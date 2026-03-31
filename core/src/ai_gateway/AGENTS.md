# AGENTS.md for core/src/ai_gateway

This directory implements the AI Gateway feature.

## Design Sources (Authoritative)

- Product intent: `../../../docs/10-prd/index.md`
- System technical design: `../../../docs/20-product-tdd/index.md`
- Core unit design: `../../../docs/30-unit-tdd/core/design.md`
- Core unit interfaces: `../../../docs/30-unit-tdd/core/interfaces.md`

The architecture guidance previously described here is now maintained in Product/Unit TDD.

## Boundary and Quality Rules

- Keep behavior aligned with interface/operation contracts in core unit TDD docs.
- Keep tests aligned under `../../tests/ai_gateway/*`.
- Keep routing deterministic and avoid hidden fallback logic.
- Keep request and stream invariants deterministic and test-covered.
