use serde_json::Value;

use crate::observability::contract::{
    ContractEvent, DescriptorCatalogChangeMode, DispatchOutcomeClass, SignalDirection,
    StemDescriptorCatalogEvent, StemDispatchTransitionEvent, StemSignalTransitionEvent,
    TransitionKind,
};

use super::{current_run_id, emit_contract_event, timestamp_now};

pub fn emit_stem_signal_transition(
    direction: SignalDirection,
    transition_kind: TransitionKind,
    descriptor_id: &str,
    endpoint_id: Option<&str>,
    sense_id: Option<&str>,
    act_id: Option<&str>,
    tick_when_known: Option<u64>,
) {
    emit_contract_event(ContractEvent::StemSignalTransition(
        StemSignalTransitionEvent {
            run_id: current_run_id().to_string(),
            timestamp: timestamp_now(),
            direction,
            transition_kind,
            descriptor_id: descriptor_id.to_string(),
            endpoint_id: endpoint_id.map(str::to_string),
            sense_id: sense_id.map(str::to_string),
            act_id: act_id.map(str::to_string),
            tick_when_known,
        },
    ));
}

pub fn emit_stem_dispatch_transition(
    act_id: &str,
    transition_kind: TransitionKind,
    queue_or_flow_summary: Value,
    tick_when_known: Option<u64>,
    terminal_outcome_when_present: Option<DispatchOutcomeClass>,
) {
    emit_contract_event(ContractEvent::StemDispatchTransition(
        StemDispatchTransitionEvent {
            run_id: current_run_id().to_string(),
            timestamp: timestamp_now(),
            act_id: act_id.to_string(),
            transition_kind,
            queue_or_flow_summary,
            tick_when_known,
            terminal_outcome_when_present,
        },
    ));
}

pub fn emit_stem_descriptor_catalog(
    catalog_version: &str,
    change_mode: DescriptorCatalogChangeMode,
    changed_descriptor_summary: Value,
    catalog_snapshot_when_required: Option<Value>,
) {
    emit_contract_event(ContractEvent::StemDescriptorCatalog(
        StemDescriptorCatalogEvent {
            run_id: current_run_id().to_string(),
            timestamp: timestamp_now(),
            catalog_version: catalog_version.to_string(),
            change_mode,
            changed_descriptor_summary,
            catalog_snapshot_when_required,
        },
    ));
}
