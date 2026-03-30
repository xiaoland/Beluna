use serde_json::Value;

use crate::observability::contract::{
    ContractEvent, DescriptorCatalogChangeMode, DispatchOutcomeClass, StemAfferentEvent,
    StemAfferentRuleEvent, StemEfferentEvent, StemNsCatalogEvent, StemProprioceptionEvent,
    StemTickEvent,
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
pub fn emit_stem_afferent(
    kind: &str,
    descriptor_id: &str,
    endpoint_id: Option<&str>,
    sense_id: Option<&str>,
    tick_when_known: Option<u64>,
    sense_payload: Option<Value>,
    weight: Option<f64>,
    queue_state: Option<Value>,
    matched_rule_ids: Option<Value>,
    reason: Option<&str>,
) {
    let tick = tick_when_known.unwrap_or(0);
    let span_seed = sense_id.or(endpoint_id).unwrap_or(descriptor_id);

    emit_contract_event(ContractEvent::StemAfferent(StemAfferentEvent {
        run_id: current_run_id().to_string(),
        timestamp: timestamp_now(),
        tick,
        span_id: format!("stem.afferent:{kind}:{span_seed}"),
        parent_span_id: None,
        kind: kind.to_string(),
        descriptor_id: descriptor_id.to_string(),
        endpoint_id: endpoint_id.map(str::to_string),
        sense_id: sense_id.map(str::to_string),
        sense_payload,
        weight,
        queue_state,
        matched_rule_ids,
        reason: reason.map(str::to_string),
    }));
}

#[allow(clippy::too_many_arguments)]
pub fn emit_stem_efferent(
    kind: &str,
    act_id: &str,
    descriptor_id: Option<&str>,
    endpoint_id: Option<&str>,
    act_payload: Option<Value>,
    queue_state: Option<Value>,
    tick_when_known: Option<u64>,
    continuity_decision: Option<&str>,
    terminal_outcome: Option<DispatchOutcomeClass>,
    reason: Option<Value>,
) {
    let tick = tick_when_known.unwrap_or(0);
    emit_contract_event(ContractEvent::StemEfferent(StemEfferentEvent {
        run_id: current_run_id().to_string(),
        timestamp: timestamp_now(),
        tick,
        span_id: format!("stem.efferent:{kind}:{act_id}"),
        parent_span_id: None,
        kind: kind.to_string(),
        act_id: act_id.to_string(),
        descriptor_id: descriptor_id.map(str::to_string),
        endpoint_id: endpoint_id.map(str::to_string),
        act_payload,
        queue_state,
        continuity_decision: continuity_decision.map(str::to_string),
        terminal_outcome,
        reason,
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

pub fn emit_stem_ns_catalog(
    tick: Option<u64>,
    catalog_version: &str,
    change_mode: DescriptorCatalogChangeMode,
    accepted_entries_or_routes: Value,
    rejected_entries_or_routes: Value,
    catalog_snapshot: Option<Value>,
) {
    emit_contract_event(ContractEvent::StemNsCatalog(StemNsCatalogEvent {
        run_id: current_run_id().to_string(),
        timestamp: timestamp_now(),
        tick: tick.unwrap_or(0),
        span_id: format!("stem.ns-catalog:{catalog_version}"),
        catalog_version: catalog_version.to_string(),
        change_mode,
        accepted_entries_or_routes,
        rejected_entries_or_routes,
        catalog_snapshot,
    }));
}

pub fn emit_stem_afferent_rule(
    tick: Option<u64>,
    kind: &str,
    revision: u64,
    rule_id: &str,
    rule: Option<Value>,
    removed: Option<bool>,
) {
    emit_contract_event(ContractEvent::StemAfferentRule(StemAfferentRuleEvent {
        run_id: current_run_id().to_string(),
        timestamp: timestamp_now(),
        tick: tick.unwrap_or(0),
        span_id: format!("stem.afferent.rule:{kind}:{rule_id}"),
        kind: kind.to_string(),
        revision,
        rule_id: rule_id.to_string(),
        rule,
        removed,
    }));
}
