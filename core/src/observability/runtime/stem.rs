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
    _kind: &str,
    _descriptor_id: &str,
    _endpoint_id: Option<&str>,
    _sense_id: Option<&str>,
    _tick_when_known: Option<u64>,
    _sense_payload: Option<Value>,
    _weight: Option<f64>,
    _queue_state: Option<Value>,
    _matched_rule_ids: Option<Value>,
    _reason: Option<&str>,
) {
}

#[allow(clippy::too_many_arguments)]
pub fn emit_stem_efferent(
    _kind: &str,
    _act_id: &str,
    _descriptor_id: Option<&str>,
    _endpoint_id: Option<&str>,
    _act_payload: Option<Value>,
    _queue_state: Option<Value>,
    _tick_when_known: Option<u64>,
    _continuity_decision: Option<&str>,
    _terminal_outcome: Option<DispatchOutcomeClass>,
    _reason: Option<Value>,
) {
}

pub fn emit_stem_proprioception(_kind: &str, _tick: Option<u64>, _entries_or_keys: Value) {}

pub fn emit_stem_ns_catalog(
    _tick: Option<u64>,
    _catalog_version: &str,
    _change_mode: DescriptorCatalogChangeMode,
    _accepted_entries_or_routes: Value,
    _rejected_entries_or_routes: Value,
    _catalog_snapshot: Option<Value>,
) {
}

pub fn emit_stem_afferent_rule(
    _tick: Option<u64>,
    _kind: &str,
    _revision: u64,
    _rule_id: &str,
    _rule: Option<Value>,
    _removed: Option<bool>,
) {
}
