# Core Observability Current State (2026-03-29)

## Goal

This report is built from direct source exploration of `core` and captures the logs that are currently emitted as structured observability contract events.

Required dimensions from the task:

- who initiates each log event
- event name (`family`)
- payload fields

## Scope

Included:

- Events emitted through `core/src/observability/runtime/emit.rs` via `emit_contract_event`.
- Structured contract logs with:
  - `target = "observability.contract"`
  - message = `"contract_event"`
  - `payload` field containing serialized `ContractEvent` JSON.

Excluded:

- Non-contract tracing/debug logs that do not serialize `ContractEvent` payload (for example `core/src/ai_gateway/telemetry.rs` events).

## Snapshot Summary

- Subsystems: `ai-gateway`, `cortex`, `stem`, `spine`
- Contract event families: `15`
- Canonical payload schema source:
  - `core/src/observability/contract/mod.rs`
- Canonical runtime emission source:
  - `core/src/observability/runtime/emit.rs`

## Deliverables

- `EVENTS-CATALOG.md`: event family catalog and payload field inventory
- `EMIT-SOURCES.md`: source-level initiator mapping (`who` + trigger path)
- `QUICK-REFERENCE.md`: fast lookup and debugging pivots

## Method

1. Enumerate all `observability_runtime::emit_*` call sites in `core/src`.
2. Trace each call to `core/src/observability/runtime/*` wrapper functions.
3. Resolve final payload schema from `ContractEvent` structs in `core/src/observability/contract/mod.rs`.
4. Confirm flattening and level rules in `core/src/observability/runtime/flatten.rs`.

## Notes

- All contract payloads are tagged with `family` (`#[serde(tag = "family")]` on `ContractEvent`).
- Log level (`info` vs `warn`) is decided by `flatten_contract_event` and not only by call site.
