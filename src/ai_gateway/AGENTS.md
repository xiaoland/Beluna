# AGENTS.md for src/ai-gateway

AI Gateway is a thin boundary that standardizes how the Beluna core calls external AI backends and receives results.

## File Structure

```text
src/ai_gateway/
├── AGENTS.md
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
├── adapters/
│   ├── mod.rs
│   ├── http_common.rs
│   ├── openai_compatible.rs
│   ├── ollama.rs
│   ├── github_copilot.rs
│   └── copilot_rpc.rs
└── tests/
    ├── mod.rs
    ├── request_normalizer.rs
    ├── router.rs
    ├── reliability.rs
    ├── budget.rs
    ├── openai_compatible.rs
    ├── ollama.rs
    ├── copilot_adapter.rs
    └── gateway_e2e.rs
```

## Design Invariants

- Keep routing deterministic: select exactly one backend, no multi-backend fallback in MVP.
- Keep protocol strict: reject invalid requests early in `RequestNormalizer`.
- Preserve canonical stream ordering:
  - first event is `Started`
  - exactly one terminal event (`Completed` or `Failed`)
- Gateway stream must not emit `ToolCallStatus::Executed` or `ToolCallStatus::Rejected`.
- Retry safety:
  - default retry only before first output/tool event
  - no hidden retry after output/tool events unless explicitly supported
- Stream drop must cancel in-flight backend work and release budget/concurrency resources.

## Adapter Rules

- BackendAdapter owns both transport and dialect mapping.
- OpenAI-compatible means `chat/completions`-like compatibility, not strict parity.
- Handle provider divergence with graceful degradation when possible.
- Map non-reconcilable payloads to deterministic canonical errors.

## Security and Credentials

- Resolve secrets only through `CredentialProvider`.
- Do not leak tokens/headers in error messages or telemetry.
- Keep redaction defaults on any provider/raw metadata output.

## Budget and Reliability

- Enforce timeout, per-backend concurrency, and rate smoothing.
- Treat usage token post-check as best-effort accounting; do not terminate active stream based on late usage.
- Circuit breaker counting should include transient backend failures, not caller cancellation.

## Testing and Docs

- Add/adjust unit tests for normalization, routing, reliability, and budget changes.
- Add/adjust adapter tests for parsing/mapping changes.

## Current State

> Last Updated At: 2026-02-08T15:04:33Z+08:00

### Live Capabilities

### Known Limitations & Mocks

- Copilot adapter targets SDK/LSP flow with conservative assumptions; method/shape drift across SDK versions may require follow-up updates.

### Immediate Next Focus
