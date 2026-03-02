# L0 Plan 02 - Sense Model and Wire Migration
- Task: `cortex-loop-architecture`
- Micro-task: `02-sense-model-and-wire-migration`
- Stage: `L0`
- Date: `2026-03-01`
- Status: `DRAFT_FOR_APPROVAL`

## Objective
Migrate sense domain contract from JSON payload + metadata object to text payload + explicit typed fields.

## Scope
1. `SenseDatum.payload` -> `String` (immediate migration).
2. Remove `SenseDatum.metadata`.
3. Add `SenseDatum.weight: f64` in `[0,1]`, default `0`.
4. Add optional `SenseDatum.act_instance_id`.
5. Define deterministic key=value metadata rendering for Primary lines.
6. Update producer/consumer wire compatibility in core, CLI, and Apple endpoint bridge.

## Current State
1. Sense payload is `serde_json::Value`.
2. Metadata is carried and validated in unix-socket adapter.
3. Weight is absent from sense model.

## Target State
1. Sense contract is text-first and strongly typed.
2. Correlation is explicit via optional `act_instance_id`.
3. Metadata rendering is derived, not stored as opaque field.

## Key Gaps
1. Core type and serde contract changes.
2. Adapter validation and parse logic changes.
3. Endpoint-side message shape migration.

## Risks
1. Breaking wire compatibility across core/CLI/apple-universal.
2. Ambiguity in key=value escaping if not strictly specified.

## L0 Exit Criteria
1. New sense schema is stable and fully enumerated.
2. Backward-compatibility stance is explicit (none required).
3. Metadata render contract has deterministic escaping/truncation rules.
