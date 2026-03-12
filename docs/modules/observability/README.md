# Observability Module

This module defines Core runtime observability conventions and cross-module telemetry correlation.

## Logging Baseline

- Runtime logging is `tracing`-only.
- Runtime logs are emitted through tracing layers; a final process-tail OTLP shutdown failure uses one stderr fallback after tracing teardown.
- `BELUNA_DEBUG_AI_GATEWAY` is removed; log verbosity is controlled by `logging.filter`.

## Logging Pipeline

Core initializes tracing at startup (`core/src/logging.rs`) with:

1. `tracing_error::ErrorLayer` for richer error context in span trees.
2. JSON file layer (local file appender):
   - includes timestamp, level, target, fields, current span, span list
   - file name format: `core.log.<YYYY-MM-DD>.<awake_sequence>`
   - `awake_sequence` is monotonic per date and increments on each process wake/start
   - retention cleanup: `logging.retention_days` (default 14)
3. stderr mirror layer:
   - enabled by `logging.stderr_warn_enabled`
   - only `warn/error` for operator visibility
4. OpenTelemetry log bridge layer:
   - forwards `tracing` events to OTLP/HTTP protobuf (`/v1/logs`)
   - configured by `observability.otlp.*`

## Metrics Pipeline

- Core uses OpenTelemetry metrics export (OTLP/HTTP protobuf, `/v1/metrics`).
- No Prometheus pull endpoint is exposed by Core.
- Current key gauges:
  - `beluna_cortex_cycle_id`
  - `beluna_cortex_input_ir_act_descriptor_catalog_count`

## Runtime Config Contract

- `logging.*` controls local file/stderr behavior.
- `observability.otlp.*` controls OTLP export:
  - `enabled`
  - `endpoint`
  - `export_timeout_ms`
  - `metrics_export_interval_ms`
  - `logs_export_interval_ms`

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
  - ai-gateway `llm_input`/`llm_output` logs

## AI Gateway Telemetry Shape

- Spans:
  - `gateway_request`
  - `gateway_stream_task`
  - adapter dispatch spans (`openai_dispatch`, `ollama_dispatch`, `copilot_dispatch`)
- LLM payload logs (ownership boundary):
  - `llm_input`
  - `llm_output`
- Event levels:
  - `info`: request/attempt lifecycle, first stream event, completion/cancellation
  - `warn`: attempt failure, request failure

## Cortex Logging Shape

- Cortex keeps IR/act-focused structured logs:
  - `input_ir_sense`
  - `input_ir_act_descriptor_catalog`
  - `output_ir_acts`
  - `final_returned_acts`
- Cortex does not log `llm_input` / `llm_output`; those are emitted by AI Gateway.
