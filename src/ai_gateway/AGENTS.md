# AGENTS.md for src/ai_gateway

This directory implements the AI Gateway feature.

## Design Sources (Authoritative)

- PRD: `docs/features/ai-gateway/PRD.md`
- HLD: `docs/features/ai-gateway/HLD.md`
- LLD: `docs/features/ai-gateway/LLD.md`

The architecture guidance previously described here is now maintained in HLD.

## Boundary and Quality Rules

- Keep behavior aligned with contracts in `docs/contracts/ai-gateway/*`.
- Keep tests aligned under `tests/ai_gateway/*`.
- Keep routing deterministic and avoid hidden fallback logic.
- Keep request and stream invariants deterministic and test-covered.

## Source Surfaces

```text
src/ai_gateway/
├── mod.rs
├── types.rs
├── error.rs
├── capabilities.rs
├── credentials.rs
├── request_normalizer.rs
├── response_normalizer.rs
├── router.rs
├── reliability.rs
├── budget.rs
├── telemetry.rs
├── gateway.rs
└── adapters/
    ├── mod.rs
    ├── http_common.rs
    ├── openai_compatible.rs
    ├── ollama.rs
    ├── github_copilot.rs
    └── copilot_rpc.rs
```

## Last Updated

> 2026-02-08
