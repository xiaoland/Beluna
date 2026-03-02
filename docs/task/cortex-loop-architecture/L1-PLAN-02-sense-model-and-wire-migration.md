# L1 Plan 02 - Sense Model and Wire Migration
- Task: `cortex-loop-architecture`
- Micro-task: `02-sense-model-and-wire-migration`
- Stage: `L1`
- Date: `2026-03-01`
- Status: `DRAFT_FOR_APPROVAL`

## High-Level Strategy
1. Execute a hard-cut schema migration for `SenseDatum` across `core`, `cli`, and `apple-universal`.
2. Standardize on text payload and typed first-class fields.
3. Remove all runtime dependence on `metadata` JSON and derive rendered metadata deterministically from typed fields.

## Architectural Design
1. Canonical sense contract becomes:
- `payload: String`
- `weight: f64` constrained to `[0, 1]` and default `0`
- `act_instance_id: Option<String>`
- remove `metadata` field.
2. Validation at ingress boundary (unix socket and inline adapters):
- reject invalid `weight`
- reject invalid `act_instance_id` format according to selected UUID policy.
3. Cortex input rendering contract is deterministic text lines:
- `- [fq-sense-id]. [key=value,key=value,...]; [payload-truncated-if-needed]`
- metadata key order is stable and globally defined.

## Key Technical Decisions
1. No backward compatibility bridge; compile breaks are used to enumerate all call sites.
2. Metadata rendering is centralized in one formatter module to prevent divergent output.
3. Deterministic escaping and float formatting are part of the contract, not implementation detail.
4. `act_instance_id` is a first-class field, never encoded inside payload or metadata blob.

## Dependency Requirements
1. Micro-task `01` DI/runtime split should land first to avoid rework in runtime entry points.
2. Formatter spec must be fixed before migrating Cortex rendering.
3. Producer migrations in CLI and Apple endpoints must be done in the same compatibility window as core parser changes.

## Maintainability Checkpoint
Text-only payload is simpler now but can reduce long-term clarity for rich structured senses. To keep maintainability acceptable, this plan assumes we keep a single centralized formatter/parser boundary and reserve future extension through explicit typed fields, not ad-hoc payload overloading.

## L1 Exit Criteria
1. One canonical `SenseDatum` contract is defined for all components.
2. Deterministic metadata rendering policy is fixed.
3. Ingress validation ownership is explicit and centralized.
