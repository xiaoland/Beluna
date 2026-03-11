# L1 Plan (High-Level Strategy)

## Accepted Decisions (From You)

1. **`./monitor` is single-port + pull mode**
   - Monitor does not receive pushed telemetry.
   - Monitor only visualizes: it pulls/scrapes/reads telemetry produced by Core.
2. **Apple Universal migrates to OpenTelemetry metrics protocol**
   - Stop parsing Prometheus text; use an OpenTelemetry-native metrics payload format.
3. **Monitor UI scope**
   - Minimal website: **raw log list** (sorting + simple filtering) + **metrics table** only.

## Problem Restatement (Operationally)

- Core must emit telemetry in a form that:
  - a local web monitor can **pull** and render,
  - and Apple Universal can **pull** metrics using an OpenTelemetry-defined payload.
- Apple Universal must **drop log observation** entirely (no directory watch, no cortex-cycle cards).

## High-Level Architecture

### Components

1. **Beluna Core (`./core`)**
   - Keep `tracing` JSON file logs as baseline (unchanged behavior).
   - Replace/augment current Prometheus metrics export with an **OpenTelemetry Metrics snapshot endpoint** (pull-friendly).
2. **Beluna Monitor (`./monitor`)**
   - Single HTTP port serving:
     - HTMX UI pages/fragments,
     - server-side rendered tables for logs + metrics.
   - Pull sources:
     - Logs: read + parse Core’s existing NDJSON file logs from `logging.dir`.
     - Metrics: pull from Core’s new OpenTelemetry metrics endpoint.
3. **Apple Universal (`./apple-universal`)**
   - Remove log observation (UI + watcher + models).
   - Keep metrics view, but fetch from Core using the OpenTelemetry metrics payload.

### Data Flow (Pull Model)

```text
Core (writes log files) ----(read file)----> Monitor (renders logs list)
Core (serves OTel metrics) --(HTTP pull)--> Monitor (renders metrics table)
Core (serves OTel metrics) --(HTTP pull)--> Apple Universal (renders pills)
```

Monitor is purely a **read-only consumer**.

## Key Technical Decisions (Proposed)

### 1) OpenTelemetry Metrics “Protocol” Choice (Needs Your Confirmation)

To keep Apple Universal lightweight (no protobuf codegen), I propose:

- Core exposes metrics as **OTLP/HTTP JSON** (proto3 JSON mapping of OTLP metric export message).
- Apple Universal parses that JSON via `Codable` for a minimal subset.

This is the most standards-aligned “OpenTelemetry metrics protocol” while keeping implementation size reasonable.

If you instead want **OTLP protobuf** (more canonical), Apple will need Swift protobuf types/codegen which will add many generated files and complexity.

**Confirm which one you want: OTLP JSON (recommended) or OTLP protobuf.**

### 2) Core Metrics Export Surface

Current Core exports Prometheus on `127.0.0.1:9464/metrics`.

Because **no backward compatibility** is required, I propose to:

- Replace the Prometheus exporter with a Core-owned HTTP endpoint, still on port **9464** by default, but serving:
  - `GET /v1/metrics` → OTLP metrics snapshot (JSON or protobuf).
  - `GET /healthz` → simple health.

This minimizes port sprawl and keeps “metrics endpoint” stable for operators (Apple already defaults to 9464).

### 3) Logs Strategy

To avoid risk from Rust OpenTelemetry “logs signal” API maturity, I propose:

- Keep Core logs as `tracing` JSON NDJSON files (existing behavior).
- Monitor reads those files and visualizes them (filter/sort).
- Apple removes all log observation.

This still provides the required **log visibility**, and Core can “embrace OpenTelemetry” primarily through **metrics**, while logs remain in the existing structured format.

If you want logs also emitted as OTLP logs (strict OpenTelemetry logs protocol), we can do it later or in this task, but it will increase implementation risk and dependency footprint.

**Confirm if “logs via existing tracing JSON files” is acceptable for this task.**

## Dependency Strategy (Minimal & Contained)

### Core

- Add a small HTTP server dependency (likely `axum` + `tower` + `hyper`, or `warp`) to host `/v1/metrics`.
- Add OpenTelemetry OTLP payload types / encoding support.
  - If **OTLP JSON**: prefer a crate path that can encode OTLP structs to JSON deterministically.
  - If **OTLP protobuf**: use `prost` encoding and set `Content-Type: application/x-protobuf`.
- Keep instrumentation call sites unchanged by preserving the `observability::metrics::*` wrapper API.

### Monitor

- Rust binary with:
  - HTTP server (`axum`),
  - HTML rendering (prefer minimal string builder or a small typed-html crate),
  - pulling metrics via `reqwest`,
  - parsing log lines via `serde_json`.

### Apple Universal

- If OTLP JSON: add Swift structs for minimal OTLP JSON subset + decoding logic.
- If OTLP protobuf: add SwiftProtobuf + generated OTLP types (higher cost).

## Compatibility & Migration Notes

- Prometheus scraping will be removed or disabled if we replace `9464/metrics`.
- Apple Universal settings UI should remove “Log Directory” controls entirely.
- Monitor becomes the canonical operator UI for logs.

## Success Criteria (L1)

- Core builds with OpenTelemetry metrics endpoint enabled.
- Monitor can display:
  - latest N logs with filter/sort,
  - metrics table showing current gauges/counters.
- Apple Universal builds and shows metrics using the OpenTelemetry metrics payload.
- Apple Universal contains **no log watching** code paths.

## Next Step

If you approve this L1 strategy, I’ll draft **L2 (Low-level Design)**:

- exact endpoints + payload shapes,
- Rust module structure,
- Apple parsing model,
- monitor routes + HTMX fragments,
- and the minimal metric set (including fixing the missing “act catalog count” gauge so the UI isn’t blank).

