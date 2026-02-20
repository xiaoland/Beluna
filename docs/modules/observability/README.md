# Observability Module

This module defines Core runtime observability conventions and cross-module telemetry correlation.

## Logging Baseline

- Runtime logging is `tracing`-only.
- No `eprintln!` fallback in runtime path.
- No `TelemetrySink` abstraction.
- `BELUNA_DEBUG_AI_GATEWAY` is removed; log verbosity is controlled by `logging.filter`.

## Logging Pipeline

Core initializes tracing at startup (`core/src/logging.rs`) with:

1. `tracing_error::ErrorLayer` for richer error context in span trees.
2. JSON file layer (rotating file appender):
   - includes timestamp, level, target, fields, current span, span list
   - rotation: `daily` or `hourly`
   - retention cleanup: `logging.retention_days` (default 14)
3. stderr mirror layer:
   - enabled by `logging.stderr_warn_enabled`
   - only `warn/error` for operator visibility

Initialization is fail-fast for invalid filter or unusable log directory.

## Context Propagation

- `main` opens one run span (`core_run`) and attaches `run_id`.
- Async task boundaries are instrumented with explicit spans (`.instrument(...)`).
- Thread boundaries capture and re-enter parent span for inline body workers.
- Gateway request handling uses request-level spans carrying `request_id`, `backend_id`, model and stage.

## Request ID Propagation

- AI Gateway request spans always include canonical `request_id`.
- HTTP adapters propagate `x-request-id` to backend requests.
- Structured events include `request_id` to correlate:
  - gateway lifecycle events
  - adapter dispatch logs
  - cortex llm input/output logs

## AI Gateway Telemetry Shape

- Spans:
  - `gateway_request`
  - `gateway_stream_task`
  - adapter dispatch spans (`openai_dispatch`, `ollama_dispatch`, `copilot_dispatch`)
- Event levels:
  - `info`: request/attempt lifecycle, first stream event, completion/cancellation
  - `warn`: attempt failure, request failure

## Recommended Filter Profiles

- Default: `info`
- Troubleshooting Gateway + Cortex:
  - `info,ai_gateway=debug,ai_gateway.openai_compatible=debug,ai_gateway.ollama=debug,ai_gateway.github_copilot=debug,cortex=debug`

## Next Enhancements

- Add OpenTelemetry exporter (optional) while keeping file logs as baseline.
- Add span latency histograms (per stage and per backend).
- Add redaction helpers for sensitive prompt/credential fields in debug mode.
