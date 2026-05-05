use serde_json::{Value, json};

use crate::observability::owner_log::{
    AdapterLifecycleState, DispatchOutcomeClass, EndpointLifecycleTransition, OwnerLogAttribute,
    OwnerLogEvent, OwnerLogSeverity, OwnerScope, canonical_scope_segment, emit,
};

pub(crate) fn emit_spine_adapter_lifecycle(
    adapter_type: &str,
    adapter_id: &str,
    kind: AdapterLifecycleState,
    reason_or_error: Option<&str>,
) {
    let event_name = adapter_lifecycle_event_name(kind);
    let adapter_segment = canonical_scope_segment(adapter_type);

    emit(OwnerLogEvent {
        scope: OwnerScope::spine_adapter(adapter_type),
        event_name,
        tick: 0,
        span_key: "adapter".to_string(),
        severity: severity_for_adapter_lifecycle(kind),
        attributes: Vec::new(),
        body: json!({
            "summary": format!("Spine adapter {event_name}."),
            "adapter_name": adapter_type,
            "adapter_segment": adapter_segment,
            "adapter_type": adapter_type,
            "adapter_id": adapter_id,
            "lifecycle_state": kind,
            "reason_or_error": reason_or_error,
        }),
    });
}

pub(crate) fn emit_spine_endpoint_lifecycle(
    endpoint_id: &str,
    adapter_id: Option<&str>,
    kind: EndpointLifecycleTransition,
    channel_or_session: Option<String>,
    route_summary: Option<Value>,
    reason_or_error: Option<&str>,
) {
    let event_name = endpoint_lifecycle_event_name(kind);
    let endpoint_segment = canonical_scope_segment(endpoint_id);

    emit(OwnerLogEvent {
        scope: OwnerScope::spine_endpoint(endpoint_id),
        event_name,
        tick: 0,
        span_key: "endpoint".to_string(),
        severity: OwnerLogSeverity::Info,
        attributes: Vec::new(),
        body: json!({
            "summary": format!("Spine endpoint {event_name}."),
            "endpoint_id": endpoint_id,
            "endpoint_segment": endpoint_segment,
            "adapter_id": adapter_id,
            "lifecycle_transition": kind,
            "channel_or_session": channel_or_session,
            "route_summary": route_summary,
            "reason_or_error": reason_or_error,
        }),
    });
}

pub(crate) fn emit_spine_sense_ingress(
    tick: u64,
    endpoint_id: &str,
    descriptor_id: Option<&str>,
    sense_id: &str,
    sense_payload: Value,
    reason: Option<&str>,
) {
    let endpoint_segment = canonical_scope_segment(endpoint_id);

    emit(OwnerLogEvent {
        scope: OwnerScope::spine_endpoint(endpoint_id),
        event_name: "sense.received",
        tick,
        span_key: format!("sense:{sense_id}"),
        severity: OwnerLogSeverity::Info,
        attributes: sense_ingress_attrs(descriptor_id),
        body: json!({
            "summary": "Spine received sense from endpoint.",
            "endpoint_id": endpoint_id,
            "endpoint_segment": endpoint_segment,
            "descriptor_id": descriptor_id,
            "sense_id": sense_id,
            "sense_payload": sense_payload,
            "reason": reason,
        }),
    });
}

pub(crate) fn emit_act_bound(
    tick: u64,
    act_id: &str,
    endpoint_id: Option<&str>,
    descriptor_id: Option<&str>,
    binding_kind: Option<&str>,
    channel_id: Option<u64>,
    act_payload: Option<Value>,
) {
    let owner_endpoint_id = endpoint_id.unwrap_or("unknown");
    let endpoint_segment = canonical_scope_segment(owner_endpoint_id);

    emit(OwnerLogEvent {
        scope: OwnerScope::spine_endpoint(owner_endpoint_id),
        event_name: "act.started",
        tick,
        span_key: format!("act:{act_id}"),
        severity: OwnerLogSeverity::Info,
        attributes: act_routing_attrs(act_id, descriptor_id),
        body: json!({
            "summary": "Spine act started.",
            "act_id": act_id,
            "endpoint_id": endpoint_id,
            "endpoint_segment": endpoint_segment,
            "descriptor_id": descriptor_id,
            "binding": {
                "kind": binding_kind,
                "channel_id": channel_id,
            },
            "act_payload": act_payload,
        }),
    });
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn emit_act_outcome(
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
    let event_name = act_outcome_event_name(outcome);
    let owner_endpoint_id = endpoint_id.unwrap_or("unknown");
    let endpoint_segment = canonical_scope_segment(owner_endpoint_id);

    emit(OwnerLogEvent {
        scope: OwnerScope::spine_endpoint(owner_endpoint_id),
        event_name,
        tick,
        span_key: format!("act:{act_id}"),
        severity: severity_for_dispatch_outcome(Some(outcome)),
        attributes: act_routing_attrs(act_id, descriptor_id),
        body: json!({
            "summary": format!("Spine act {event_name}."),
            "act_id": act_id,
            "endpoint_id": endpoint_id,
            "endpoint_segment": endpoint_segment,
            "descriptor_id": descriptor_id,
            "binding": {
                "kind": binding_kind,
                "channel_id": channel_id,
            },
            "outcome": outcome,
            "reason_or_reference": reason_or_reference,
            "act_payload": act_payload,
        }),
    });
}

fn adapter_lifecycle_event_name(kind: AdapterLifecycleState) -> &'static str {
    match kind {
        AdapterLifecycleState::Enabled => "enabled",
        AdapterLifecycleState::Disabled => "disabled",
        AdapterLifecycleState::Faulted => "faulted",
    }
}

fn endpoint_lifecycle_event_name(kind: EndpointLifecycleTransition) -> &'static str {
    match kind {
        EndpointLifecycleTransition::Connected => "connected",
        EndpointLifecycleTransition::Disconnected => "disconnected",
        EndpointLifecycleTransition::Registered => "registered",
        EndpointLifecycleTransition::Dropped => "dropped",
    }
}

fn act_outcome_event_name(outcome: DispatchOutcomeClass) -> &'static str {
    match outcome {
        DispatchOutcomeClass::Acknowledged => "act.finished",
        DispatchOutcomeClass::Rejected => "act.rejected",
        DispatchOutcomeClass::Lost => "act.lost",
    }
}

fn severity_for_adapter_lifecycle(kind: AdapterLifecycleState) -> OwnerLogSeverity {
    match kind {
        AdapterLifecycleState::Faulted => OwnerLogSeverity::Error,
        _ => OwnerLogSeverity::Info,
    }
}

fn severity_for_dispatch_outcome(outcome: Option<DispatchOutcomeClass>) -> OwnerLogSeverity {
    match outcome {
        Some(DispatchOutcomeClass::Lost) => OwnerLogSeverity::Error,
        Some(DispatchOutcomeClass::Rejected) => OwnerLogSeverity::Warn,
        _ => OwnerLogSeverity::Info,
    }
}

fn sense_ingress_attrs(descriptor_id: Option<&str>) -> Vec<OwnerLogAttribute> {
    descriptor_id
        .map(|descriptor_id| {
            vec![OwnerLogAttribute::string(
                "spine.descriptor.id",
                descriptor_id,
            )]
        })
        .unwrap_or_default()
}

fn act_routing_attrs(act_id: &str, descriptor_id: Option<&str>) -> Vec<OwnerLogAttribute> {
    let mut attributes = vec![OwnerLogAttribute::string("spine.act.id", act_id)];
    if let Some(descriptor_id) = descriptor_id {
        attributes.push(OwnerLogAttribute::string(
            "spine.descriptor.id",
            descriptor_id,
        ));
    }
    attributes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_spine_runtime_kinds_to_event_names() {
        assert_eq!(
            adapter_lifecycle_event_name(AdapterLifecycleState::Enabled),
            "enabled"
        );
        assert_eq!(
            endpoint_lifecycle_event_name(EndpointLifecycleTransition::Registered),
            "registered"
        );
        assert_eq!(
            act_outcome_event_name(DispatchOutcomeClass::Acknowledged),
            "act.finished"
        );
        assert_eq!(
            act_outcome_event_name(DispatchOutcomeClass::Rejected),
            "act.rejected"
        );
        assert_eq!(
            act_outcome_event_name(DispatchOutcomeClass::Lost),
            "act.lost"
        );
    }
}
