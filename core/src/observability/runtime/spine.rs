use crate::observability::contract::{
    AdapterLifecycleState, ContractEvent, DispatchOutcomeClass, EndpointLifecycleTransition,
    SpineAdapterEvent, SpineDispatchEvent, SpineEndpointEvent,
};

use super::{current_run_id, emit_contract_event, timestamp_now};

pub fn emit_spine_adapter_lifecycle(
    adapter_type: &str,
    adapter_id: &str,
    state_transition: AdapterLifecycleState,
    reason_or_error_when_present: Option<&str>,
) {
    emit_contract_event(ContractEvent::SpineAdapter(SpineAdapterEvent {
        run_id: current_run_id().to_string(),
        timestamp: timestamp_now(),
        tick: 0,
        span_id: format!("spine.adapter:{adapter_type}:{adapter_id}"),
        adapter_type: adapter_type.to_string(),
        adapter_id: adapter_id.to_string(),
        kind_or_state: state_transition,
        reason_or_error_when_present: reason_or_error_when_present.map(str::to_string),
    }));
}

pub fn emit_spine_endpoint_lifecycle(
    endpoint_id: &str,
    transition_kind: EndpointLifecycleTransition,
    channel_or_session_when_present: Option<String>,
    reason_or_error_when_present: Option<&str>,
) {
    emit_contract_event(ContractEvent::SpineEndpoint(SpineEndpointEvent {
        run_id: current_run_id().to_string(),
        timestamp: timestamp_now(),
        tick: 0,
        span_id: format!("spine.endpoint:{endpoint_id}"),
        endpoint_id: endpoint_id.to_string(),
        kind_or_transition: transition_kind,
        adapter_id_when_present: None,
        channel_or_session_when_present,
        route_summary_when_present: None,
        reason_or_error_when_present: reason_or_error_when_present.map(str::to_string),
    }));
}

pub fn emit_spine_dispatch_bind(
    tick: u64,
    act_id: &str,
    endpoint_id: &str,
    descriptor_id_when_present: Option<&str>,
    binding_kind_when_present: Option<&str>,
    channel_id_when_present: Option<u64>,
) {
    emit_contract_event(ContractEvent::SpineDispatch(SpineDispatchEvent {
        run_id: current_run_id().to_string(),
        timestamp: timestamp_now(),
        tick,
        span_id: format!("spine.dispatch.bind:{act_id}"),
        parent_span_id_when_present: None,
        act_id: act_id.to_string(),
        endpoint_id: endpoint_id.to_string(),
        descriptor_id_when_present: descriptor_id_when_present.map(str::to_string),
        kind: "bind".to_string(),
        binding_kind_when_present: binding_kind_when_present.map(str::to_string),
        channel_id_when_present,
        outcome_when_present: None,
        reason_code_when_present: None,
        reference_id_when_present: None,
    }));
}

#[allow(clippy::too_many_arguments)]
pub fn emit_spine_dispatch_outcome(
    tick: u64,
    act_id: &str,
    endpoint_id: &str,
    descriptor_id_when_present: Option<&str>,
    binding_kind_when_present: Option<&str>,
    channel_id_when_present: Option<u64>,
    outcome_when_present: DispatchOutcomeClass,
    reason_code_when_present: Option<&str>,
    reference_id_when_present: Option<&str>,
) {
    emit_contract_event(ContractEvent::SpineDispatch(SpineDispatchEvent {
        run_id: current_run_id().to_string(),
        timestamp: timestamp_now(),
        tick,
        span_id: format!("spine.dispatch.outcome:{act_id}"),
        parent_span_id_when_present: None,
        act_id: act_id.to_string(),
        endpoint_id: endpoint_id.to_string(),
        descriptor_id_when_present: descriptor_id_when_present.map(str::to_string),
        kind: "outcome".to_string(),
        binding_kind_when_present: binding_kind_when_present.map(str::to_string),
        channel_id_when_present,
        outcome_when_present: Some(outcome_when_present),
        reason_code_when_present: reason_code_when_present.map(str::to_string),
        reference_id_when_present: reference_id_when_present.map(str::to_string),
    }));
}
