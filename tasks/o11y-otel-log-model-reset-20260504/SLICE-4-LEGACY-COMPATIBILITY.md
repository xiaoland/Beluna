# Slice 4: Legacy Compatibility Projection

Status: implemented in Moira Lachesis.

## MVT Anchors

- Objective & Hypothesis: keep historical `ContractEvent`-shaped local logs inspectable while native owner logs become the primary Beluna telemetry model.
- Guardrails Touched: Core emission semantics stay Core-owned; Moira compatibility markers stay Moira-local and do not redefine the target OTLP contract.
- Verification: Lachesis stores `record_kind`, raw drilldown shows it, native records keep scope/event/trace/span identity, and legacy payload records remain readable through compatibility normalization.

## Implemented Shape

1. `raw_events` now carries `record_kind`.
2. Current values:
   - `native_owner`: `scope.name` starts with `beluna.core.` and `eventName` is present.
   - `legacy_contract`: `scope.name` is `observability.contract`, or the record carries legacy `family` plus serialized `payload`.
   - `ordinary_log`: all other records.
3. Backend queries derive a fallback `record_kind` for existing local rows that predate the column.
4. Frontend `RawEvent` preserves `recordKind` and uses the same fallback classification when older bridge payloads omit it.
5. Raw event drilldown exposes `record_kind` in both the summary metadata and the OTLP JSON section.

## Test Shape

The backend trace-backed store test remains because it guards the native tick anchor and selected detail read path.

The frontend native timeline projection fixture was lowered to raw event normalization. That keeps the test aligned with the stable data boundary while leaving future timeline reconstruction free to change.

