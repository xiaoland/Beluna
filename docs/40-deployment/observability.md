# Observability

## Ownership

Core is the single owner of runtime observability export.

## Signals

1. Logs: tracing-based JSON file logs and optional OTLP logs export.
2. Metrics: OTLP metrics export.
3. Traces: OTLP traces export with configurable sampling ratio.

## Constraints

1. No Prometheus pull endpoint in current contract.
2. Body endpoints do not duplicate Core observability control surfaces.
3. OTLP signal configuration is explicit and per-signal.

Decision Source: ADR-001.
