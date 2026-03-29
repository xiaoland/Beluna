# Core Observability

This file defines Core's local OTLP log family catalog and the logical fixture contract used by Core-side event-shape tests.

Cross-unit reconstruction guarantees belong in `docs/20-product-tdd/observability-contract.md`.
Loom composition and operator-facing inspection surfaces belong in `docs/30-unit-tdd/moira/*`.

## Local Rules

1. OTLP family naming is Core-owned and grouped by Core subsystem.
2. Fixture records normalize emitted OTLP events into logical Beluna fields for contract testing; OTLP transport wrapper details may be tested separately.
3. Every fixture must include `family`, `run_id`, `timestamp`, and the correlation keys needed to reconstruct the event.
4. Tick-bound fixtures use `tick`, not `cycle_id`.
5. Large payload families may expose summaries plus stable references or handles instead of duplicating full bodies in every fixture.
6. Golden fixture bundles live under `core/tests/fixtures/observability/` and are validated by `core/tests/observability_contract.rs`.

## Family Spec Table

| Family | Emit point | Required logical fields | Supports |
|---|---|---|---|
| `cortex.tick` | Once per admitted, inspectable tick | `run_id`; `tick`; `trigger_summary`; `senses_summary`; `proprioception_snapshot_or_ref`; `acts_summary`; `goal_forest_ref` | cycle reconstruction; goal-forest linkage |
| `cortex.organ.request` | When an operator-relevant primary request crosses a Cortex organ or tool boundary | `run_id`; `tick`; `stage`; `route_or_organ`; `request_id`; `input_summary` | request/response correlation inside one tick |
| `cortex.organ.response` | For each matching operator-relevant request | `run_id`; `tick`; `stage`; `request_id`; `status`; `response_summary`; `tool_summary`; `act_summary`; `error_summary_when_present` | request/response correlation inside one tick |
| `cortex.goal_forest.snapshot` | Once per inspectable tick in the current contract | `run_id`; `tick`; `snapshot_summary`; `snapshot_or_ref` | per-tick goal-forest reconstruction |
| `stem.signal.transition` | For afferent or efferent lifecycle transitions that matter to flow understanding | `run_id`; `direction`; `transition_kind`; `descriptor_id`; `endpoint_id_when_present`; `sense_id_when_present`; `act_id_when_present`; `tick_when_known` | signal-flow reconstruction |
| `stem.dispatch.transition` | For queueing or dispatch transitions that materially change dispatch state before terminal outcome | `run_id`; `act_id`; `transition_kind`; `queue_or_flow_summary`; `tick_when_known`; `terminal_outcome_when_present` | act-to-dispatch path reconstruction |
| `stem.descriptor.catalog` | At run bootstrap and whenever descriptor catalog version changes | `run_id`; `catalog_version`; `change_mode`; `changed_descriptor_summary`; `catalog_snapshot_when_required` | descriptor history reconstruction |
| `spine.adapter.lifecycle` | On adapter enable, disable, or fault lifecycle changes | `run_id`; `adapter_type`; `adapter_id`; `state_transition`; `reason_or_error_when_present` | adapter topology reconstruction |
| `spine.endpoint.lifecycle` | On endpoint connect, disconnect, register, or drop lifecycle changes | `run_id`; `endpoint_id`; `channel_or_session_when_present`; `transition_kind`; `reason_or_error_when_present` | endpoint topology reconstruction |
| `spine.dispatch.outcome` | For every terminal dispatch outcome | `run_id`; `act_id`; `binding_target`; `descriptor_id_when_present`; `outcome`; `latency_ms_when_present`; `tick_when_known` | dispatch-outcome reconstruction |

## Minimum Fixture Set

| Family | Required fixture coverage |
|---|---|
| `cortex.tick` | nominal tick with non-empty senses and acts; minimal tick with empty summaries |
| `cortex.organ.request` | nominal primary request |
| `cortex.organ.response` | matched nominal response; response with explicit error summary |
| `cortex.goal_forest.snapshot` | inline snapshot fixture; stable-reference snapshot fixture |
| `stem.signal.transition` | afferent transition fixture; efferent transition fixture |
| `stem.dispatch.transition` | queue-state transition fixture; transition carrying terminal outcome when present |
| `stem.descriptor.catalog` | bootstrap snapshot fixture; incremental update or drop fixture |
| `spine.adapter.lifecycle` | enable fixture; disable or fault fixture |
| `spine.endpoint.lifecycle` | connect or register fixture; disconnect or drop fixture |
| `spine.dispatch.outcome` | `Acknowledged` fixture; `Rejected` or `Lost` fixture |

## Change Discipline

1. Renaming a family or changing its required logical field set requires updating this file and the corresponding Core fixtures in the same change.
2. If a change affects only Core-local naming or optional debug fields, keep it in Core Unit TDD unless it changes a cross-unit reconstruction guarantee.
3. If a new family is needed only to improve Core-local diagnosability, Product TDD does not need to change.
4. If a new family or field is required so Moira can reconstruct a new surface, update Product TDD first and then update this file.
