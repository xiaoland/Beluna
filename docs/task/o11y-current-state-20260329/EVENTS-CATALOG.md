# Events Catalog (Contract Payload)

This catalog is source-of-truth from `core/src/observability/contract/mod.rs` and runtime value flow in `core/src` call sites.

## Unified Contract Log Envelope

All contract events are emitted by `core/src/observability/runtime/emit.rs` with:

- target: `observability.contract`
- message: `contract_event`
- flattened attributes: `subsystem`, `family`, `run_id`, `tick`, `organ_id`, `thread_id`, `turn_id`, `request_id`, `endpoint_id`, `descriptor_id`, `act_id`, `sense_id`, `adapter_id`, `adapter_type`, `transition_kind`, `outcome`, `direction`, `binding_kind`, `change_mode`, `state`, `kind`, `payload`

`payload` is serialized `ContractEvent` JSON (tagged by `family`).

## Family: ai-gateway.request

Payload required fields:

- `run_id`, `timestamp`, `tick`, `request_id`, `span_id`, `backend_id`, `model`, `kind`, `input_payload`, `enable_thinking`

Payload optional fields:

- `parent_span_id_when_present`, `organ_id_when_present`, `thread_id_when_present`, `turn_id_when_present`, `attempt_when_present`, `effective_tools_when_present`, `limits_when_present`, `provider_request_when_present`, `provider_response_when_present`, `usage_when_present`, `error_when_present`

Current value domain (from call sites):

- `kind`: `start`, `attempt_failed`, `succeeded`, `failed`

Warn level condition (flatten):

- `error_when_present` is present

## Family: ai-gateway.turn

Payload required fields:

- `run_id`, `timestamp`, `tick`, `thread_id`, `turn_id`, `span_id`, `status`, `metadata`

Payload optional fields:

- `parent_span_id_when_present`, `organ_id_when_present`, `request_id_when_present`, `messages_when_committed`, `finish_reason_when_present`, `usage_when_present`, `backend_metadata_when_present`, `error_when_present`

Current value domain:

- `status`: `error`, `committed`, `committed_pending_continuation`

Warn level condition:

- `status == "error"`

## Family: ai-gateway.thread

Payload required fields:

- `run_id`, `timestamp`, `tick`, `thread_id`, `span_id`, `kind`, `messages`

Payload optional fields:

- `parent_span_id_when_present`, `organ_id_when_present`, `turn_summaries_when_present`, `source_turn_ids_when_present`

Current value domain:

- `kind`: `opened`, `cloned`, `turn_committed`

Warn level condition:

- none (always info in flatten)

## Family: cortex.tick

Payload required fields:

- `run_id`, `timestamp`, `tick`, `span_id`, `kind_or_status`, `drained_senses`, `physical_state_snapshot`

Payload optional fields:

- `tick_seq_when_present`, `control_gate_state_when_present`, `acts_payload_or_summary_when_present`, `goal_forest_snapshot_ref_or_payload_when_present`, `error_when_present`

Current value domain:

- `kind_or_status`: `ok`, `primary_contract_error`

Warn level condition:

- `error_when_present` is present

## Family: cortex.organ

Payload required fields:

- `run_id`, `timestamp`, `tick`, `organ_id`, `request_id`, `span_id`, `phase`, `status`

Payload optional fields:

- `parent_span_id_when_present`, `route_or_backend_when_present`, `input_payload_when_present`, `output_payload_when_present`, `error_when_present`, `ai_gateway_request_id_when_present`, `thread_id_when_present`, `turn_id_when_present`

Current value domain:

- `phase`: `start`, `end`
- `status`: `ok`, `error`

Warn level condition:

- `status == error`

## Family: cortex.goal-forest

Payload required fields:

- `run_id`, `timestamp`, `tick`, `span_id`, `kind`

Payload optional fields:

- `parent_span_id_when_present`, `snapshot_when_present`, `patch_request_when_present`, `patch_result_when_present`, `cognition_persisted_revision_when_present`, `reset_context_applied_when_present`, `selected_turn_ids_when_present`

Current value domain:

- `kind`: `snapshot`, `patch`

Warn level condition:

- none (always info in flatten)

## Family: stem.tick

Payload required fields:

- `run_id`, `timestamp`, `tick`, `span_id`, `status`, `tick_seq`

Payload optional fields:

- none

Current value domain:

- `status`: `granted`

Warn level condition:

- none (always info in flatten)

## Family: stem.signal

Payload required fields:

- `run_id`, `timestamp`, `tick`, `span_id`, `direction`, `transition_kind`, `descriptor_id`

Payload optional fields:

- `parent_span_id_when_present`, `endpoint_id_when_present`, `sense_id_when_present`, `act_id_when_present`, `sense_payload_when_present`, `act_payload_when_present`, `weight_when_present`, `queue_or_deferred_state_when_present`, `matched_rule_ids_when_present`, `reason_when_present`

Current value domain:

- `direction`: `afferent`, `efferent`
- `transition_kind`: `enqueue`, `defer`, `release`, `dispatch`, `result`
- current `reason_when_present`: used on efferent `result` path (`ACK`, `REJECTED`, `LOST` text)

Warn level condition:

- `reason_when_present` is present

## Family: stem.dispatch

Payload required fields:

- `run_id`, `timestamp`, `tick`, `span_id`, `act_id`, `kind`, `queue_or_flow_summary`

Payload optional fields:

- `parent_span_id_when_present`, `descriptor_id_when_present`, `endpoint_id_when_present`, `act_payload_when_present`, `continuity_decision_when_present`, `terminal_outcome_when_present`, `reason_or_reference_when_present`

Current value domain:

- `kind`: `enqueue`, `dispatch`, `result`
- `terminal_outcome_when_present`: `acknowledged`, `rejected`, `lost` (result path)

Warn level condition:

- `terminal_outcome_when_present` is `rejected` or `lost`

## Family: stem.proprioception

Payload required fields:

- `run_id`, `timestamp`, `tick`, `span_id`, `kind`, `entries_or_keys`

Payload optional fields:

- none

Current value domain:

- `kind`: `patch`, `drop`

Warn level condition:

- none (always info in flatten)

## Family: stem.descriptor.catalog

Payload required fields:

- `run_id`, `timestamp`, `tick`, `span_id`, `catalog_version`, `change_mode`, `accepted_entries_or_routes`, `rejected_entries_or_routes`

Payload optional fields:

- `catalog_snapshot_when_required`

Current value domain:

- `change_mode`: `snapshot`, `update`, `drop`

Warn level condition:

- none (always info in flatten)

## Family: stem.afferent.rule

Payload required fields:

- `run_id`, `timestamp`, `tick`, `span_id`, `kind`, `revision`, `rule_id`

Payload optional fields:

- `rule_when_present`, `removed_when_present`

Current value domain:

- `kind`: `add`, `remove`

Warn level condition:

- none (always info in flatten)

## Family: spine.adapter

Payload required fields:

- `run_id`, `timestamp`, `tick`, `span_id`, `adapter_type`, `adapter_id`, `kind_or_state`

Payload optional fields:

- `reason_or_error_when_present`

Current value domain:

- `kind_or_state`: `enabled`, `faulted`

Warn level condition:

- `kind_or_state == faulted`

## Family: spine.endpoint

Payload required fields:

- `run_id`, `timestamp`, `tick`, `span_id`, `endpoint_id`, `kind_or_transition`

Payload optional fields:

- `adapter_id_when_present`, `channel_or_session_when_present`, `route_summary_when_present`, `reason_or_error_when_present`

Current value domain:

- `kind_or_transition`: `connected`, `dropped`

Warn level condition:

- none (always info in flatten)

## Family: spine.dispatch

Payload required fields:

- `run_id`, `timestamp`, `tick`, `span_id`, `act_id`, `endpoint_id`, `kind`

Payload optional fields:

- `parent_span_id_when_present`, `descriptor_id_when_present`, `binding_kind_when_present`, `channel_id_when_present`, `outcome_when_present`, `reason_code_when_present`, `reference_id_when_present`

Current value domain:

- `kind`: `bind`, `outcome`
- `binding_kind_when_present`: `inline`, `adapter`
- `outcome_when_present`: `acknowledged`, `rejected`, `lost` (outcome path)

Warn level condition:

- `outcome_when_present` is `rejected` or `lost`
