# Observability

## Ownership

Core is the single owner of runtime observability export.
Moira is the default first-party local control-plane and observability consumer for operator-facing log inspection.

## Signals

1. Logs: tracing-based JSON file logs and optional OTLP logs export. Cross-unit reconstruction guarantees for local first-party observability are defined in `docs/20-product-tdd/observability-contract.md`. Local operator workflow targets Moira/Loom as the primary first-party inspection surface.
2. Metrics: OTLP metrics export. Local control-plane workflow surfaces exporter status and handoff destinations rather than first-party local metrics storage.
3. Traces: OTLP traces export with configurable sampling ratio. Local control-plane workflow surfaces exporter status and handoff destinations rather than first-party local trace storage.

## Constraints

1. No Prometheus pull endpoint in current contract.
2. Body endpoints and Moira do not duplicate Core observability emission authority.
3. OTLP signal configuration is explicit and per-signal.
4. Local observability storage/query consumers must preserve Core-owned signal semantics instead of redefining them.
