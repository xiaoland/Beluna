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

## Core Runtime Interface Contracts

1. Afferent ingress accepts domain senses with descriptor identity.
2. Tick grants control admitted Cortex cycle execution.
3. Act dispatch returns one terminal outcome per act.
4. Continuity persists and restores cognition state with guardrails.
