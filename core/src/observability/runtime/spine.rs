use serde_json::Value;

use crate::observability::contract::{
    AdapterLifecycleState, ContractEvent, DispatchOutcomeClass, EndpointLifecycleTransition,
    SpineActEvent, SpineAdapterEvent, SpineEndpointEvent, SpineSenseEvent,
};

use super::{current_run_id, emit_contract_event, timestamp_now};

pub fn emit_spine_adapter_lifecycle(
    adapter_type: &str,
    adapter_id: &str,
    kind: AdapterLifecycleState,
    reason_or_error: Option<&str>,
) {
    emit_contract_event(ContractEvent::SpineAdapter(SpineAdapterEvent {
        run_id: current_run_id().to_string(),
        timestamp: timestamp_now(),
        tick: 0,
        span_id: format!("spine.adapter:{adapter_type}:{adapter_id}"),
        adapter_type: adapter_type.to_string(),
        adapter_id: adapter_id.to_string(),
        kind,
        reason_or_error: reason_or_error.map(str::to_string),
    }));
}

pub fn emit_spine_endpoint_lifecycle(
    endpoint_id: &str,
    adapter_id: Option<&str>,
    kind: EndpointLifecycleTransition,
    channel_or_session: Option<String>,
    route_summary: Option<Value>,
    reason_or_error: Option<&str>,
) {
    emit_contract_event(ContractEvent::SpineEndpoint(SpineEndpointEvent {
        run_id: current_run_id().to_string(),
        timestamp: timestamp_now(),
        tick: 0,
        span_id: format!("spine.endpoint:{endpoint_id}"),
        endpoint_id: endpoint_id.to_string(),
        adapter_id: adapter_id.map(str::to_string),
        kind,
        channel_or_session,
        route_summary,
        reason_or_error: reason_or_error.map(str::to_string),
    }));
}

pub fn emit_spine_sense_ingress(
    tick: u64,
    endpoint_id: &str,
    descriptor_id: Option<&str>,
    sense_id: &str,
    sense_payload: Value,
    reason: Option<&str>,
) {
    emit_contract_event(ContractEvent::SpineSense(SpineSenseEvent {
        run_id: current_run_id().to_string(),
        timestamp: timestamp_now(),
        tick,
        span_id: format!("spine.sense:ingress:{sense_id}"),
        parent_span_id: None,
        endpoint_id: endpoint_id.to_string(),
        descriptor_id: descriptor_id.map(str::to_string),
        sense_id: sense_id.to_string(),
        kind: "ingress".to_string(),
        sense_payload,
        reason: reason.map(str::to_string),
    }));
}

pub fn emit_spine_act_bind(
    tick: u64,
    act_id: &str,
    endpoint_id: Option<&str>,
    descriptor_id: Option<&str>,
    binding_kind: Option<&str>,
    channel_id: Option<u64>,
    act_payload: Option<Value>,
) {
    emit_contract_event(ContractEvent::SpineAct(SpineActEvent {
        run_id: current_run_id().to_string(),
        timestamp: timestamp_now(),
        tick,
        span_id: format!("spine.act:bind:{act_id}"),
        parent_span_id: None,
        act_id: act_id.to_string(),
        endpoint_id: endpoint_id.map(str::to_string),
        descriptor_id: descriptor_id.map(str::to_string),
        kind: "bind".to_string(),
        binding_kind: binding_kind.map(str::to_string),
        channel_id,
        act_payload,
        outcome: None,
        reason_or_reference: None,
    }));
}

#[allow(clippy::too_many_arguments)]
pub fn emit_spine_act_outcome(
    tick: u64,
    act_id: &str,
    endpoint_id: Option<&str>,
    descriptor_id: Option<&str>,
    binding_kind: Option<&str>,
    channel_id: Option<u64>,
    act_payload: Option<Value>,
    outcome: DispatchOutcomeClass,
    reason_or_reference: Option<Value>,
) {
    emit_contract_event(ContractEvent::SpineAct(SpineActEvent {
        run_id: current_run_id().to_string(),
        timestamp: timestamp_now(),
        tick,
        span_id: format!("spine.act:outcome:{act_id}"),
        parent_span_id: None,
        act_id: act_id.to_string(),
        endpoint_id: endpoint_id.map(str::to_string),
        descriptor_id: descriptor_id.map(str::to_string),
        kind: "outcome".to_string(),
        binding_kind: binding_kind.map(str::to_string),
        channel_id,
        act_payload,
        outcome: Some(outcome),
        reason_or_reference,
    }));
}
