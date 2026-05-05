use serde_json::Value;

use crate::observability::owner_log;

use super::{AdapterLifecycleState, DispatchOutcomeClass, EndpointLifecycleTransition};

pub fn emit_spine_adapter_lifecycle(
    _adapter_type: &str,
    _adapter_id: &str,
    _kind: AdapterLifecycleState,
    _reason_or_error: Option<&str>,
) {
}

pub fn emit_spine_endpoint_lifecycle(
    _endpoint_id: &str,
    _adapter_id: Option<&str>,
    _kind: EndpointLifecycleTransition,
    _channel_or_session: Option<String>,
    _route_summary: Option<Value>,
    _reason_or_error: Option<&str>,
) {
}

pub fn emit_spine_sense_ingress(
    _tick: u64,
    _endpoint_id: &str,
    _descriptor_id: Option<&str>,
    _sense_id: &str,
    _sense_payload: Value,
    _reason: Option<&str>,
) {
}

pub fn emit_spine_act_bind(
    _tick: u64,
    _act_id: &str,
    _endpoint_id: Option<&str>,
    _descriptor_id: Option<&str>,
    _binding_kind: Option<&str>,
    _channel_id: Option<u64>,
    _act_payload: Option<Value>,
) {
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
    owner_log::events::emit_act_delivered(
        tick,
        act_id,
        endpoint_id,
        descriptor_id,
        binding_kind,
        act_payload.clone(),
        outcome,
        reason_or_reference.clone(),
    );

    let _ = channel_id;
}
