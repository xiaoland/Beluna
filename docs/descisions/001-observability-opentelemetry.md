# 001 - Observability OpenTelemetry Migration

Date: 2026-03-12

## Status

Accepted

## Context

Core previously exported runtime metrics via a Prometheus pull endpoint, while Apple Universal embedded additional in-chat observability surfaces (metrics polling, log directory watching, cortex-cycle cards).

This created split observability ownership and duplicated operator-facing concerns in the desktop endpoint.

## Decision

1. Core adopts OpenTelemetry OTLP/HTTP as the observability export contract for metrics and logs.
2. Core keeps local JSON file logs as baseline runtime logs and dual-writes logs to OTLP.
3. Prometheus exporter and `/metrics` pull endpoint are removed from Core.
4. Apple Universal removes all observability surfaces (metrics endpoint controls, metrics pills, log watching, cortex-cycle cards) and focuses on body-endpoint chat responsibilities only.
5. OTLP endpoint and export timing are explicit Core config (`observability.otlp.*`) and validated by schema.

## Consequences

- Core is now the single owner of runtime observability export.
- Operators should consume telemetry from OTLP receiver/collector instead of Prometheus scraping Core directly.
- Apple Universal implementation and settings surface become smaller and easier to maintain.
- Existing app-side observability settings in `UserDefaults` are intentionally not preserved.
