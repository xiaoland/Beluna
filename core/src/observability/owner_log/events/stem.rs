use serde_json::{Value, json};

use crate::observability::owner_log::{
    DescriptorCatalogChangeMode, DispatchOutcomeClass, OwnerLogEvent, OwnerLogSeverity, OwnerScope,
    emit,
};

#[allow(clippy::too_many_arguments)]
pub(crate) fn emit_stem_afferent_pathway(
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
    let event_name = stem_afferent_event_name(kind);

    emit(OwnerLogEvent {
        scope: OwnerScope::StemAfferentPathway,
        event_name,
        tick: pre_tick_or_known_tick(tick_when_known),
        span_key: sense_span_key(sense_id, descriptor_id),
        severity: severity_for_pathway_event(event_name),
        attributes: Vec::new(),
        body: json!({
            "summary": format!("Stem afferent pathway {event_name}."),
            "pathway_event_kind": kind,
            "sense_id": sense_id,
            "endpoint_id": endpoint_id,
            "descriptor_id": descriptor_id,
            "tick_when_known": tick_when_known,
            "sense_payload": sense_payload,
            "weight": weight,
            "queue_state": queue_state,
            "matched_rule_ids": matched_rule_ids,
            "reason": reason,
        }),
    });
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn emit_stem_efferent_pathway(
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
    let event_name = stem_efferent_event_name(kind);

    emit(OwnerLogEvent {
        scope: OwnerScope::StemEfferentPathway,
        event_name,
        tick: pre_tick_or_known_tick(tick_when_known),
        span_key: format!("act:{act_id}"),
        severity: severity_for_dispatch_outcome(terminal_outcome),
        attributes: Vec::new(),
        body: json!({
            "summary": format!("Stem efferent pathway {event_name}."),
            "pathway_event_kind": kind,
            "act_id": act_id,
            "endpoint_id": endpoint_id,
            "descriptor_id": descriptor_id,
            "tick_when_known": tick_when_known,
            "act_payload": act_payload,
            "queue_state": queue_state,
            "continuity_decision": continuity_decision,
            "terminal_outcome": terminal_outcome,
            "reason": reason,
        }),
    });
}

pub(crate) fn emit_stem_proprioception(kind: &str, tick: Option<u64>, entries_or_keys: Value) {
    let event_name = proprioception_event_name(kind);

    emit(OwnerLogEvent {
        scope: OwnerScope::StemProprioception,
        event_name,
        tick: pre_tick_or_known_tick(tick),
        span_key: "state".to_string(),
        severity: OwnerLogSeverity::Info,
        attributes: Vec::new(),
        body: json!({
            "summary": format!("Stem proprioception {event_name}."),
            "proprioception_event_kind": kind,
            "tick_when_known": tick,
            "change": entries_or_keys,
        }),
    });
}

pub(crate) fn emit_stem_descriptor_catalog(
    tick: Option<u64>,
    catalog_version: &str,
    change_mode: DescriptorCatalogChangeMode,
    accepted_entries_or_routes: Value,
    rejected_entries_or_routes: Value,
    catalog_snapshot: Option<Value>,
) {
    let event_name = descriptor_catalog_event_name(change_mode);

    emit(OwnerLogEvent {
        scope: OwnerScope::StemDescriptorCatalog,
        event_name,
        tick: pre_tick_or_known_tick(tick),
        span_key: format!("version:{catalog_version}"),
        severity: OwnerLogSeverity::Info,
        attributes: Vec::new(),
        body: json!({
            "summary": format!("Stem descriptor catalog {event_name}."),
            "catalog_version": catalog_version,
            "change_mode": change_mode,
            "tick_when_known": tick,
            "accepted": accepted_entries_or_routes,
            "rejected": rejected_entries_or_routes,
            "snapshot": catalog_snapshot,
        }),
    });
}

pub(crate) fn emit_stem_afferent_rule(
    tick: Option<u64>,
    kind: &str,
    revision: u64,
    rule_id: &str,
    rule: Option<Value>,
    removed: Option<bool>,
) {
    let event_name = afferent_rule_event_name(kind);

    emit(OwnerLogEvent {
        scope: OwnerScope::StemAfferentPathway,
        event_name,
        tick: pre_tick_or_known_tick(tick),
        span_key: format!("rule:{rule_id}"),
        severity: OwnerLogSeverity::Info,
        attributes: Vec::new(),
        body: json!({
            "summary": format!("Stem afferent pathway {event_name}."),
            "rule_event_kind": kind,
            "revision": revision,
            "rule_id": rule_id,
            "rule": rule,
            "removed": removed,
            "tick_when_known": tick,
        }),
    });
}

fn stem_afferent_event_name(kind: &str) -> &'static str {
    match kind {
        "enqueue" => "sense.enqueued",
        "defer" => "sense.deferred",
        "drop" => "sense.dropped",
        "release" => "sense.released",
        _ => "sense.observed",
    }
}

fn stem_efferent_event_name(kind: &str) -> &'static str {
    match kind {
        "enqueue" => "act.enqueued",
        "dispatch" => "act.started",
        "result" => "act.finished",
        "drop" => "act.dropped",
        _ => "act.observed",
    }
}

fn proprioception_event_name(kind: &str) -> &'static str {
    match kind {
        "patch" => "patched",
        "drop" => "dropped",
        _ => "updated",
    }
}

fn descriptor_catalog_event_name(change_mode: DescriptorCatalogChangeMode) -> &'static str {
    match change_mode {
        DescriptorCatalogChangeMode::Snapshot => "snapshot",
        DescriptorCatalogChangeMode::Update => "updated",
        DescriptorCatalogChangeMode::Drop => "dropped",
    }
}

fn afferent_rule_event_name(kind: &str) -> &'static str {
    match kind {
        "add" => "rules.added",
        "remove" => "rules.removed",
        "replace" => "rules.replaced",
        _ => "rules.updated",
    }
}

fn severity_for_pathway_event(event_name: &str) -> OwnerLogSeverity {
    match event_name {
        "sense.dropped" => OwnerLogSeverity::Warn,
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

fn pre_tick_or_known_tick(tick: Option<u64>) -> u64 {
    tick.unwrap_or(0)
}

fn sense_span_key(sense_id: Option<&str>, descriptor_id: &str) -> String {
    match sense_id {
        Some(sense_id) => format!("sense:{sense_id}"),
        None => format!("descriptor:{descriptor_id}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_stem_runtime_kinds_to_event_names() {
        assert_eq!(stem_afferent_event_name("enqueue"), "sense.enqueued");
        assert_eq!(stem_afferent_event_name("defer"), "sense.deferred");
        assert_eq!(stem_afferent_event_name("drop"), "sense.dropped");
        assert_eq!(stem_afferent_event_name("release"), "sense.released");
        assert_eq!(stem_efferent_event_name("dispatch"), "act.started");
        assert_eq!(stem_efferent_event_name("result"), "act.finished");
        assert_eq!(afferent_rule_event_name("replace"), "rules.replaced");
    }
}
