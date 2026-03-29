# Core Observability

This file defines Core's Stage 2 local OTLP log model and the logical fixture contract used by Core-side event-shape checks.

Cross-unit reconstruction guarantees belong in `docs/20-product-tdd/observability-contract.md`.
Loom composition and operator-facing inspection surfaces belong in `docs/30-unit-tdd/moira/*`.

## Local Rules

1. OTLP family naming is Core-owned and grouped by the runtime owners that actually mutate or observe the state: `ai-gateway`, `cortex`, `stem`, and `spine`.
2. Stage 2 favors fewer owner-centric logical families with richer `kind`, `status`, and `transition_kind` fields over a large per-verb family lattice.
3. Every canonical emitted event includes `family`, `run_id`, and `timestamp`. `tick` is required for tick-scoped runtime activity; bootstrap or pre-first-grant activity is normalized to `tick = 0`.
4. Events that participate in within-tick causality include `span_id`. `parent_span_id` is required whenever the event is nested under another within-tick operation rather than being a root span.
5. Family naming and lane identity are separate concerns. Owner-centric families such as `spine.dispatch` or `ai-gateway.request` are valid event families, but lane typing must remain entity-centric.
6. Stronger runtime anchors such as `request_id`, `thread_id`, `turn_id`, `organ_id`, `sense_id`, `act_id`, `endpoint_id`, and `channel_id` are preferred over opaque span-only grouping whenever the runtime naturally owns them.
7. Tick-bound canonical export fields use `tick`, not `cycle_id`. Internal Core names may remain `cycle_id` until broader refactors justify churn.
8. During Beluna's early development phase, Core preserves full request, response, signal, and topology payloads in raw OTLP events by default. Summary fields may supplement these payloads but must not replace them.
9. When Cortex invokes the AI gateway, the originating `organ_id` is required on the related AI-gateway events so Moira can align LLM activity with Cortex lanes inside one tick.
10. Goal-forest observability must reflect the mutation semantics the runtime actually owns today. The stable Stage 2 mutation surface is patch/persist-oriented, not a speculative botanical verb catalog.
11. Golden fixture bundles live under `core/tests/fixtures/observability/` and are refreshed only after the family catalog stabilizes enough to justify the maintenance cost.

## Logical Family Table

These are the logical families Moira may rely on. During implementation, a logical family may temporarily appear as one coarse family with `kind` fields or as a small suffix split such as `request/response`, as long as the same owner-centric semantics and required fields remain intact.

| Logical family | Runtime owner / emit point | Required logical fields | Supports |
|---|---|---|---|
| `ai-gateway.request` | `ChatRuntime::dispatch_complete()` and its retry/resilience loop | `run_id`; `tick`; `request_id`; `span_id`; `parent_span_id_when_present`; `organ_id_when_present`; `thread_id_when_present`; `turn_id_when_present`; `backend_id`; `model`; `kind`; `attempt_when_present`; `input_payload`; `effective_tools_when_present`; `limits_when_present`; `enable_thinking`; `provider_request_when_present`; `provider_response_when_present`; `usage_when_present`; `error_when_present`; `resource_kind`; `resource_id` | backend request and retry inspection |
| `ai-gateway.turn` | `Thread::complete()` at committed-turn or terminal-failure boundary | `run_id`; `tick`; `thread_id`; `turn_id`; `span_id`; `parent_span_id_when_present`; `organ_id_when_present`; `request_id_when_present`; `status`; `messages_when_committed`; `metadata`; `finish_reason_when_present`; `usage_when_present`; `backend_metadata_when_present`; `error_when_present`; `resource_kind`; `resource_id` | committed conversation inspection |
| `ai-gateway.thread` | `Chat::open_thread()`, `Chat::clone_thread_with_turns()`, and persisted thread rewrites after completed turns | `run_id`; `tick`; `thread_id`; `span_id`; `parent_span_id_when_present`; `organ_id_when_present`; `kind`; `messages`; `turn_summaries_when_present`; `source_turn_ids_when_present`; `resource_kind`; `resource_id` | authoritative thread reconstruction |
| `cortex.tick` | `CortexRuntime::on_tick()` plus post-primary settlement | `run_id`; `tick`; `span_id`; `kind_or_status`; `tick_seq_when_present`; `drained_senses`; `physical_state_snapshot`; `control_gate_state_when_present`; `acts_payload_or_summary_when_present`; `goal_forest_snapshot_ref_or_payload_when_present`; `error_when_present`; `resource_kind`; `resource_id` | admitted, skipped, and completed tick narrative |
| `cortex.organ` | `run_primary_turn()` and `run_organ()` | `run_id`; `tick`; `organ_id`; `request_id`; `span_id`; `parent_span_id_when_present`; `route_or_backend_when_present`; `phase`; `input_payload_when_present`; `output_payload_when_present`; `status`; `error_when_present`; `ai_gateway_request_id_when_present`; `thread_id_when_present`; `turn_id_when_present`; `resource_kind`; `resource_id` | organ boundary inspection |
| `cortex.goal-forest` | tick snapshot emission and primary goal-forest patch/persist path | `run_id`; `tick`; `span_id`; `parent_span_id_when_present`; `kind`; `snapshot_when_present`; `patch_request_when_present`; `patch_result_when_present`; `cognition_persisted_revision_when_present`; `reset_context_applied_when_present`; `selected_turn_ids_when_present`; `resource_kind`; `resource_id` | goal-forest state and mutation narrative |
| `stem.tick` | `StemTickRuntime` grant loop | `run_id`; `tick`; `span_id`; `status`; `tick_seq`; `resource_kind`; `resource_id` | Stem rhythm and bootstrap anchor |
| `stem.signal` | afferent and efferent signal transition emit points | `run_id`; `tick`; `span_id`; `parent_span_id_when_present`; `direction`; `transition_kind`; `descriptor_id`; `endpoint_id_when_present`; `sense_id_when_present`; `act_id_when_present`; `sense_payload_when_present`; `act_payload_when_present`; `weight_when_present`; `queue_or_deferred_state_when_present`; `matched_rule_ids_when_present`; `reason_when_present`; `resource_kind`; `resource_id` | signal movement and loss inspection |
| `stem.dispatch` | efferent queue admission, dispatch, and terminal result | `run_id`; `tick`; `span_id`; `parent_span_id_when_present`; `act_id`; `descriptor_id_when_present`; `endpoint_id_when_present`; `kind`; `act_payload`; `queue_or_flow_summary`; `continuity_decision_when_present`; `terminal_outcome_when_present`; `reason_or_reference_when_present`; `resource_kind`; `resource_id` | dispatch queue and terminal outcome inspection |
| `stem.proprioception` | `StemPhysicalStateStore` patch/drop mutation boundary | `run_id`; `tick`; `span_id`; `kind`; `entries_or_keys`; `resource_kind`; `resource_id` | proprioception history |
| `stem.descriptor.catalog` | `StemPhysicalStateStore` catalog snapshot/update/drop commit | `run_id`; `tick`; `span_id`; `catalog_version`; `change_mode`; `accepted_entries_or_routes`; `rejected_entries_or_routes`; `catalog_snapshot_when_required`; `resource_kind`; `resource_id` | descriptor catalog history |
| `stem.afferent.rule` | afferent scheduler rule add/remove boundary | `run_id`; `tick`; `span_id`; `kind`; `revision`; `rule_id`; `rule_when_present`; `removed_when_present`; `resource_kind`; `resource_id` | deferral-rule lifecycle |
| `spine.adapter` | adapter startup and fault handling | `run_id`; `tick`; `span_id`; `adapter_type`; `adapter_id`; `kind_or_state`; `reason_or_error_when_present`; `resource_kind`; `resource_id` | adapter topology reconstruction |
| `spine.endpoint` | endpoint connect/drop and equivalent lifecycle changes | `run_id`; `tick`; `span_id`; `endpoint_id`; `kind_or_transition`; `channel_or_session_when_present`; `route_summary_when_present`; `reason_or_error_when_present`; `resource_kind`; `resource_id` | endpoint topology reconstruction |
| `spine.dispatch` | dispatch resolution and terminal outcome logging | `run_id`; `tick`; `span_id`; `parent_span_id_when_present`; `act_id`; `endpoint_id`; `descriptor_id_when_present`; `kind`; `binding_kind_when_present`; `channel_id_when_present`; `outcome_when_present`; `reason_code_when_present`; `reference_id_when_present`; `resource_kind`; `resource_id` | dispatch binding and outcome reconstruction |

## Lane Resolution Contract

Loom resolves chronology lanes in two steps:

1. Choose an entity-centric `lane_type` from the event family group.
2. Choose a `lane_key` using the highest-priority stable identity that the event exposes.

Family names are not lane types. For example, `spine.dispatch` remains a valid family name, but its primary lane type is `act`, not `spine.dispatch`.

| Event family group | Lane type | Lane-key priority |
|---|---|---|
| `cortex.organ` | `organ` | `organ_id` > `request_id` > `span_id` |
| `ai-gateway.request`, `ai-gateway.turn`, `ai-gateway.thread` | `thread` | `thread_id` > `turn_id` > `request_id` > `span_id` |
| afferent `stem.signal` | `sense` | `sense_id` > `endpoint_id` > `span_id` |
| efferent `stem.signal`, `stem.dispatch` | `act` | `act_id` > `endpoint_id` > `span_id` |
| `spine.endpoint` | `endpoint` | `endpoint_id` > `adapter_id` > `span_id` |
| `spine.dispatch` | `act` | `act_id` > `endpoint_id` > `span_id` |

Implementation note:

1. `ai-gateway.request` is the correct logical family name for the backend-governed request and retry lifecycle. Reliability, adapter behavior, and retry detail belong in `kind`, `attempt`, `error`, and related payload fields rather than in a more specific family name.
2. Stage 2 guarantees `provider_request_when_present` on terminal `ai-gateway.request` kinds when an adapter cannot expose clean intermediate bodies. Adapters may emit the same payload on earlier kinds when that is cheap and semantically honest.
3. If a future view needs a secondary grouping inside one lane, that is a Loom concern layered on top of this primary lane contract rather than a reason to make families less readable.

## Minimum Fixture Set

When the logical family catalog is refreshed into fixtures, the minimum coverage should include:

1. `ai-gateway.*`
- request success with full request/response payloads
- request retry or failure with attempt metadata
- one committed turn with user, assistant, tool-call, and tool-result messages
- one thread snapshot after a completed turn
- one thread snapshot after thread clone or reset-style rewrite

2. `cortex.*`
- one skipped or gated tick
- one admitted and completed tick with full senses plus physical snapshot
- one organ boundary with full input and output
- one goal-forest snapshot
- one goal-forest patch/persist event with context-reset details when applicable

3. `stem.*`
- one `stem.tick`
- one afferent signal transition with full sense payload
- one afferent deferral or release path
- one efferent queue/dispatch/result path with full act payload
- one proprioception patch and one drop
- one descriptor catalog update or drop
- one afferent-rule add or remove

4. `spine.*`
- one adapter lifecycle change
- one endpoint lifecycle change
- one dispatch bind
- one terminal dispatch outcome

## Change Discipline

1. Renaming a logical family or changing its required logical field set requires updating this file and the corresponding Core emit points in the same change.
2. If one logical family is emitted through temporary suffix splits such as `request/response` or `bind/outcome`, the split must still preserve the same owner-centric semantics and required correlation fields.
3. If a change affects only Core-local debug decoration and not the canonical reconstruction fields, keep it in Core Unit TDD unless it changes a cross-unit guarantee.
4. If a new field or family is required so Moira can reconstruct a new surface, update Product TDD first and then update this file.
5. Do not split one logical family into more poetic or more granular verb families unless the runtime already owns and emits those semantics. Fewer honest families with richer payloads are preferred.
