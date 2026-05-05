# Core Interfaces

## External Interfaces

1. CLI entrypoint:
- `beluna [--config <path>]`.

2. Body endpoint integration:
- UnixSocket NDJSON protocol for external endpoints.
- Inline adapter contract for built-in endpoints.

3. Configuration interface:
- Single JSONC input (`beluna.jsonc`) validated through typed config boundary.
- Schema generation via CLI command (`beluna config schema`).

4. Observability export interface:
- OTLP logs satisfy the cross-unit reconstruction guarantees defined in `docs/20-product-tdd/observability-contract.md`.
- The operator-facing cognition-cycle anchor is reconstructed from native `traceId` plus bootstrap and tick anchor events.
- Current owner scope, `eventName`, span-key, body, and attribute contract is defined in [Observability](./observability.md).

## Core Runtime Interface Contracts

1. Afferent ingress accepts domain senses with descriptor identity.
2. Tick grants control admitted Cortex cycle execution.
3. Act dispatch returns one terminal outcome per act.
4. Continuity persists and restores cognition state with guardrails.
