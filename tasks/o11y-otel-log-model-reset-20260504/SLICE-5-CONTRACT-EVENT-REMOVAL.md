# Slice 5: ContractEvent Runtime Removal

Status: implemented in Core runtime.

## MVT Anchors

- Objective & Hypothesis: retire the central `ContractEvent` algebra from the Core runtime emission path so first-party observability is owned by native OTLP owner-log schemas.
- Guardrails Touched: public runtime wrapper names remain stable for existing callsites; the first native event surface stays limited to the eight confirmed event classes.
- Verification: Core tests pass, `ContractEvent`, `emit_contract_event()`, and `flatten_contract_event()` are absent from `core/src`, and the remaining event classification enums live with owner-log schema types.

## Implemented Shape

1. Removed `core/src/observability/contract`.
2. Removed `core/src/observability/runtime/emit.rs`.
3. Removed `core/src/observability/runtime/flatten.rs`.
4. Moved runtime observability classification enums into `owner_log::schema` and re-exported them through `observability::runtime`.
5. Kept existing runtime wrapper functions as the callsite boundary.
6. Wrapper functions emit native owner logs for the current eight event classes.
7. Wrapper functions for later owner-specific event classes keep their signatures and currently produce no first-party log record.

## Follow-On Boundary

The remaining work is additive owner event expansion:

1. Stem afferent/efferent/proprioception/catalog/rule events.
2. Spine adapter/endpoint/sense/bind events.
3. Cortex helper and goal-forest events.
4. AI Gateway chat thread lifecycle events.
