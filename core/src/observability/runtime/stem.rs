use serde_json::Value;

use crate::observability::owner_log;

use super::{DescriptorCatalogChangeMode, DispatchOutcomeClass};

pub fn emit_stem_tick(tick: u64, tick_seq: u64, status: &str) {
    if status == "granted" {
        owner_log::events::emit_tick_granted(tick, tick_seq);
    }
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
    owner_log::events::emit_stem_afferent_pathway(
        kind,
        descriptor_id,
        endpoint_id,
        sense_id,
        tick_when_known,
        sense_payload,
        weight,
        queue_state,
        matched_rule_ids,
        reason,
    );
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
    owner_log::events::emit_stem_efferent_pathway(
        kind,
        act_id,
        descriptor_id,
        endpoint_id,
        act_payload,
        queue_state,
        tick_when_known,
        continuity_decision,
        terminal_outcome,
        reason,
    );
}

pub fn emit_stem_proprioception(kind: &str, tick: Option<u64>, entries_or_keys: Value) {
    owner_log::events::emit_stem_proprioception(kind, tick, entries_or_keys);
}

pub fn emit_stem_ns_catalog(
    tick: Option<u64>,
    catalog_version: &str,
    change_mode: DescriptorCatalogChangeMode,
    accepted_entries_or_routes: Value,
    rejected_entries_or_routes: Value,
    catalog_snapshot: Option<Value>,
) {
    owner_log::events::emit_stem_descriptor_catalog(
        tick,
        catalog_version,
        change_mode,
        accepted_entries_or_routes,
        rejected_entries_or_routes,
        catalog_snapshot,
    );
}

pub fn emit_stem_afferent_rule(
    tick: Option<u64>,
    kind: &str,
    revision: u64,
    rule_id: &str,
    rule: Option<Value>,
    removed: Option<bool>,
) {
    owner_log::events::emit_stem_afferent_rule(tick, kind, revision, rule_id, rule, removed);
}
