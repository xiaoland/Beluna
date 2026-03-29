use crate::observability::contract::{
    AdapterLifecycleState, ContractEvent, DispatchOutcomeClass, EndpointLifecycleTransition,
    SpineAdapterLifecycleEvent, SpineDispatchOutcomeEvent, SpineEndpointLifecycleEvent,
};

use super::{current_run_id, emit_contract_event, timestamp_now};

pub fn emit_spine_adapter_lifecycle(
    adapter_type: &str,
    adapter_id: &str,
    state_transition: AdapterLifecycleState,
    reason_or_error_when_present: Option<&str>,
) {
    emit_contract_event(ContractEvent::SpineAdapterLifecycle(
        SpineAdapterLifecycleEvent {
            run_id: current_run_id().to_string(),
            timestamp: timestamp_now(),
            adapter_type: adapter_type.to_string(),
            adapter_id: adapter_id.to_string(),
            state_transition,
            reason_or_error_when_present: reason_or_error_when_present.map(str::to_string),
        },
    ));
}

pub fn emit_spine_endpoint_lifecycle(
    endpoint_id: &str,
    transition_kind: EndpointLifecycleTransition,
    channel_or_session_when_present: Option<String>,
    reason_or_error_when_present: Option<&str>,
) {
    emit_contract_event(ContractEvent::SpineEndpointLifecycle(
        SpineEndpointLifecycleEvent {
            run_id: current_run_id().to_string(),
            timestamp: timestamp_now(),
            endpoint_id: endpoint_id.to_string(),
            transition_kind,
            channel_or_session_when_present,
            reason_or_error_when_present: reason_or_error_when_present.map(str::to_string),
        },
    ));
}

pub fn emit_spine_dispatch_outcome(
    act_id: &str,
    binding_target: &str,
    outcome: DispatchOutcomeClass,
    descriptor_id_when_present: Option<&str>,
    latency_ms_when_present: Option<u64>,
    tick_when_known: Option<u64>,
) {
    emit_contract_event(ContractEvent::SpineDispatchOutcome(
        SpineDispatchOutcomeEvent {
            run_id: current_run_id().to_string(),
            timestamp: timestamp_now(),
            act_id: act_id.to_string(),
            binding_target: binding_target.to_string(),
            outcome,
            descriptor_id_when_present: descriptor_id_when_present.map(str::to_string),
            latency_ms_when_present,
            tick_when_known,
        },
    ));
}
