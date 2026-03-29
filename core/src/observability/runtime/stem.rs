use serde_json::Value;

use crate::observability::contract::{
    ContractEvent, DescriptorCatalogChangeMode, DispatchOutcomeClass, SignalDirection,
    StemAfferentRuleEvent, StemDescriptorCatalogEvent, StemDispatchEvent, StemProprioceptionEvent,
    StemSignalEvent, StemTickEvent, TransitionKind,
};

use super::{current_run_id, emit_contract_event, timestamp_now};

pub fn emit_stem_tick(tick: u64, tick_seq: u64, status: &str) {
    emit_contract_event(ContractEvent::StemTick(StemTickEvent {
        run_id: current_run_id().to_string(),
        timestamp: timestamp_now(),
        tick,
        span_id: format!("stem.tick:{tick_seq}"),
        status: status.to_string(),
        tick_seq,
    }));
}

#[allow(clippy::too_many_arguments)]
pub fn emit_stem_signal_transition(
    direction: SignalDirection,
    transition_kind: TransitionKind,
    descriptor_id: &str,
    endpoint_id_when_present: Option<&str>,
    sense_id_when_present: Option<&str>,
    act_id_when_present: Option<&str>,
    tick_when_known: Option<u64>,
    sense_payload_when_present: Option<Value>,
    act_payload_when_present: Option<Value>,
    weight_when_present: Option<f64>,
    queue_or_deferred_state_when_present: Option<Value>,
    matched_rule_ids_when_present: Option<Value>,
    reason_when_present: Option<&str>,
) {
    let tick = tick_when_known.unwrap_or(0);
    let span_seed = sense_id_when_present
        .or(act_id_when_present)
        .or(endpoint_id_when_present)
        .unwrap_or(descriptor_id);

    emit_contract_event(ContractEvent::StemSignal(StemSignalEvent {
        run_id: current_run_id().to_string(),
        timestamp: timestamp_now(),
        tick,
        span_id: format!(
            "stem.signal:{}:{span_seed}",
            label_transition(transition_kind)
        ),
        parent_span_id_when_present: None,
        direction,
        transition_kind,
        descriptor_id: descriptor_id.to_string(),
        endpoint_id_when_present: endpoint_id_when_present.map(str::to_string),
        sense_id_when_present: sense_id_when_present.map(str::to_string),
        act_id_when_present: act_id_when_present.map(str::to_string),
        sense_payload_when_present,
        act_payload_when_present,
        weight_when_present,
        queue_or_deferred_state_when_present,
        matched_rule_ids_when_present,
        reason_when_present: reason_when_present.map(str::to_string),
    }));
}

#[allow(clippy::too_many_arguments)]
pub fn emit_stem_dispatch_transition(
    act_id: &str,
    descriptor_id_when_present: Option<&str>,
    endpoint_id_when_present: Option<&str>,
    kind: &str,
    act_payload_when_present: Option<Value>,
    queue_or_flow_summary: Value,
    tick_when_known: Option<u64>,
    continuity_decision_when_present: Option<&str>,
    terminal_outcome_when_present: Option<DispatchOutcomeClass>,
    reason_or_reference_when_present: Option<Value>,
) {
    let tick = tick_when_known.unwrap_or(0);
    emit_contract_event(ContractEvent::StemDispatch(StemDispatchEvent {
        run_id: current_run_id().to_string(),
        timestamp: timestamp_now(),
        tick,
        span_id: format!("stem.dispatch:{kind}:{act_id}"),
        parent_span_id_when_present: None,
        act_id: act_id.to_string(),
        descriptor_id_when_present: descriptor_id_when_present.map(str::to_string),
        endpoint_id_when_present: endpoint_id_when_present.map(str::to_string),
        kind: kind.to_string(),
        act_payload_when_present,
        queue_or_flow_summary,
        continuity_decision_when_present: continuity_decision_when_present.map(str::to_string),
        terminal_outcome_when_present,
        reason_or_reference_when_present,
    }));
}

pub fn emit_stem_proprioception(kind: &str, tick: Option<u64>, entries_or_keys: Value) {
    emit_contract_event(ContractEvent::StemProprioception(StemProprioceptionEvent {
        run_id: current_run_id().to_string(),
        timestamp: timestamp_now(),
        tick: tick.unwrap_or(0),
        span_id: format!("stem.proprioception:{kind}"),
        kind: kind.to_string(),
        entries_or_keys,
    }));
}

pub fn emit_stem_descriptor_catalog(
    tick: Option<u64>,
    catalog_version: &str,
    change_mode: DescriptorCatalogChangeMode,
    accepted_entries_or_routes: Value,
    rejected_entries_or_routes: Value,
    catalog_snapshot_when_required: Option<Value>,
) {
    emit_contract_event(ContractEvent::StemDescriptorCatalog(
        StemDescriptorCatalogEvent {
            run_id: current_run_id().to_string(),
            timestamp: timestamp_now(),
            tick: tick.unwrap_or(0),
            span_id: format!("stem.descriptor.catalog:{catalog_version}"),
            catalog_version: catalog_version.to_string(),
            change_mode,
            accepted_entries_or_routes,
            rejected_entries_or_routes,
            catalog_snapshot_when_required,
        },
    ));
}

pub fn emit_stem_afferent_rule(
    tick: Option<u64>,
    kind: &str,
    revision: u64,
    rule_id: &str,
    rule_when_present: Option<Value>,
    removed_when_present: Option<bool>,
) {
    emit_contract_event(ContractEvent::StemAfferentRule(StemAfferentRuleEvent {
        run_id: current_run_id().to_string(),
        timestamp: timestamp_now(),
        tick: tick.unwrap_or(0),
        span_id: format!("stem.afferent.rule:{kind}:{rule_id}"),
        kind: kind.to_string(),
        revision,
        rule_id: rule_id.to_string(),
        rule_when_present,
        removed_when_present,
    }));
}

fn label_transition(kind: TransitionKind) -> &'static str {
    match kind {
        TransitionKind::Enqueue => "enqueue",
        TransitionKind::Defer => "defer",
        TransitionKind::Release => "release",
        TransitionKind::Dispatch => "dispatch",
        TransitionKind::Result => "result",
    }
}
