# L0 Plan (Analysis & Context)

## Task Statement (Deconstructed)

1. **Core embraces OpenTelemetry**
   - Export **logs** and **metrics** via OpenTelemetry (OTLP) so downstream tooling can observe Core activity without tailing local files.
2. **Minimal monitoring website**
   - Add a new `./monitor` component that:
     - Listens on a **localhost port**.
     - Receives Core telemetry **through OpenTelemetry** (OTLP).
     - Provides a **minimal HTMX website** to view **logs + metrics**.
3. **Apple Universal app**
   - Remove **logs observation** (log directory watching + cortex cycle cards).
   - Keep **metrics observation**.

## Current Repository Reality (What Exists Today)

### `./core` (Rust)

- Logging:
  - `tracing` JSON file logs with retention and optional stderr mirroring.
  - Init code: `core/src/logging.rs` (`init_tracing`).
  - File naming: `core.log.<YYYY-MM-DD>.<awake_sequence>`.
- Metrics:
  - Uses the `metrics` crate + `metrics-exporter-prometheus`.
  - Prometheus endpoint: `http://127.0.0.1:9464/metrics`.
  - Init code: `core/src/observability/metrics.rs` + `core/src/main.rs`.
- Observability docs already anticipate OTel:
  - `docs/modules/observability/README.md` has “Next Enhancements: Add OpenTelemetry exporter (optional) while keeping file logs as baseline.”
- No OpenTelemetry dependencies present in the repo today (no `opentelemetry*` crates).

### `./apple-universal` (Swift)

- Metrics observation:
  - Polls Prometheus endpoint and parses a small set of gauges (string parsing).
  - UI shows pill metrics in `ChatView`.
- Logs observation (to be removed):
  - Watches Core log directory via `DispatchSource` and parses NDJSON log files.
  - Pairs `cortex_organ_input` + `cortex_organ_output` into “Cortex Cycle” cards.
  - Settings include “Log Directory” controls and status.

## Constraints / Invariants (From AGENTS.md)

- Prefer **high cohesion / low coupling**.
- **No backward compatibility required**, but avoid needless churn.
- **Build should pass** (tests aren’t required to be maintained/run).
- Core rule: avoid implicit fallbacks masking missing state.

## Internet Research Status

- Firecrawl is available but currently **blocked** (“Insufficient credits”), so I cannot fetch up-to-date OpenTelemetry Rust docs in this environment.
- Plan assumes implementation will be guided by:
  - local compilation feedback (`cargo build`),
  - small, incremental dependency additions,
  - minimal surface area changes.

## Architectural Options & Trade-offs

### A) OTLP over HTTP (single-port monitor)

- Core exports OTLP/HTTP protobuf to `http://127.0.0.1:<port>/{v1/logs,v1/metrics}`.
- `./monitor` serves:
  - OTLP receiver endpoints (`/v1/logs`, `/v1/metrics`) **and**
  - HTMX UI (`/`) on the same port.
- Pros: simplest local ergonomics; one process/port.
- Cons: Rust OTel OTLP/HTTP support + log pipeline API stability may vary by crate versions.

### B) OTLP over gRPC (standard ports)

- Core exports OTLP/gRPC to `127.0.0.1:4317`.
- `./monitor` runs:
  - a gRPC OTLP receiver on `4317`
  - and a separate HTTP UI server on `3030` (or similar).
- Pros: OTLP/gRPC is the “default” in many stacks; sometimes more stable.
- Cons: two ports; slightly more moving parts; more code to glue together.

### C) Run an OpenTelemetry Collector (external) + UI

- `./monitor` would be mainly the UI; collector config lives in repo.
- Pros: standards-compliant; less custom OTLP decoding.
- Cons: introduces a new external runtime dependency (collector binary/docker).

## Key Open Questions (Need Your Decision)

1. **Protocol/ports**
   - Do you prefer **single-port** `monitor` (OTLP/HTTP + UI together), or is **two-port** (OTLP gRPC + UI) acceptable?
2. **Prometheus compatibility**
   - Should Core **keep** the existing Prometheus `/metrics` endpoint for Apple Universal (least change),
     or should Apple Universal switch to reading metrics from `monitor` (cleaner, but bigger change)?
3. **UI scope**
   - “Minimal website” = raw log list + metric table only,
     or should it also reintroduce the “Cortex Cycle” grouped view (like the Apple app) to preserve that workflow?

## Provisional Direction (If You Don’t Object)

- Keep existing file logs + Prometheus as baseline.
- Add optional OpenTelemetry exporters in Core (logs + metrics) gated by config.
- Implement `./monitor` as a minimal OTLP receiver + HTMX UI (single-port HTTP) unless you prefer gRPC.
- Remove all log watching + cortex-cycle UI from Apple Universal; keep metrics pills/polling.

