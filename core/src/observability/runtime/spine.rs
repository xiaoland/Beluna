use serde_json::Value;

use crate::observability::owner_log;

use super::{AdapterLifecycleState, DispatchOutcomeClass, EndpointLifecycleTransition};

pub fn emit_spine_adapter_lifecycle(
    adapter_type: &str,
    adapter_id: &str,
    kind: AdapterLifecycleState,
    reason_or_error: Option<&str>,
) {
    owner_log::events::emit_spine_adapter_lifecycle(
        adapter_type,
        adapter_id,
        kind,
        reason_or_error,
    );
}

pub fn emit_spine_endpoint_lifecycle(
    endpoint_id: &str,
    adapter_id: Option<&str>,
    kind: EndpointLifecycleTransition,
    channel_or_session: Option<String>,
    route_summary: Option<Value>,
    reason_or_error: Option<&str>,
) {
    owner_log::events::emit_spine_endpoint_lifecycle(
        endpoint_id,
        adapter_id,
        kind,
        channel_or_session,
        route_summary,
        reason_or_error,
    );
}

pub fn emit_spine_sense_ingress(
    tick: u64,
    endpoint_id: &str,
    descriptor_id: Option<&str>,
    sense_id: &str,
    sense_payload: Value,
    reason: Option<&str>,
) {
    owner_log::events::emit_spine_sense_ingress(
        tick,
        endpoint_id,
        descriptor_id,
        sense_id,
        sense_payload,
        reason,
    );
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
    owner_log::events::emit_act_bound(
        tick,
        act_id,
        endpoint_id,
        descriptor_id,
        binding_kind,
        channel_id,
        act_payload,
    );
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
    owner_log::events::emit_act_outcome(
        tick,
        act_id,
        endpoint_id,
        descriptor_id,
        binding_kind,
        channel_id,
        act_payload.clone(),
        outcome,
        reason_or_reference.clone(),
    );
}
