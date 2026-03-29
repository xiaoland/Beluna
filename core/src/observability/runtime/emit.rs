use super::flatten::{EventLevel, FlatContractEvent, flatten_contract_event};
use crate::observability::contract::ContractEvent;

pub(crate) fn emit_contract_event(event: ContractEvent) {
    let flat = flatten_contract_event(&event);
    let payload = serde_json::to_string(&event)
        .unwrap_or_else(|err| format!("{{\"serialization_error\":\"{err}\"}}"));
    log_flat_event(flat, payload);
}

fn log_flat_event(flat: FlatContractEvent, payload: String) {
    let FlatContractEvent {
        level,
        subsystem,
        family,
        run_id,
        tick,
        organ_id,
        thread_id,
        turn_id,
        request_id,
        endpoint_id,
        descriptor_id,
        act_id,
        sense_id,
        adapter_id,
        adapter_type,
        transition_kind,
        outcome,
        direction,
        binding_kind,
        change_mode,
        state,
        kind,
    } = flat;

    let tick_present = tick > 0;
    let organ_id = organ_id.as_deref().unwrap_or("");
    let thread_id = thread_id.as_deref().unwrap_or("");
    let turn_id = turn_id.as_deref().unwrap_or("");
    let request_id = request_id.as_deref().unwrap_or("");
    let endpoint_id = endpoint_id.as_deref().unwrap_or("");
    let descriptor_id = descriptor_id.as_deref().unwrap_or("");
    let act_id = act_id.as_deref().unwrap_or("");
    let sense_id = sense_id.as_deref().unwrap_or("");
    let adapter_id = adapter_id.as_deref().unwrap_or("");
    let adapter_type = adapter_type.as_deref().unwrap_or("");
    let transition_kind = transition_kind.as_deref().unwrap_or("");
    let outcome = outcome.as_deref().unwrap_or("");
    let direction = direction.as_deref().unwrap_or("");
    let binding_kind = binding_kind.as_deref().unwrap_or("");
    let change_mode = change_mode.as_deref().unwrap_or("");
    let state = state.as_deref().unwrap_or("");
    let kind = kind.as_deref().unwrap_or("");

    match level {
        EventLevel::Info => tracing::info!(
            target: "observability.contract",
            subsystem = subsystem,
            family = family,
            run_id = %run_id,
            tick = tick,
            tick_present = tick_present,
            organ_id = organ_id,
            thread_id = thread_id,
            turn_id = turn_id,
            request_id = request_id,
            endpoint_id = endpoint_id,
            descriptor_id = descriptor_id,
            act_id = act_id,
            sense_id = sense_id,
            adapter_id = adapter_id,
            adapter_type = adapter_type,
            transition_kind = transition_kind,
            outcome = outcome,
            direction = direction,
            binding_kind = binding_kind,
            change_mode = change_mode,
            state = state,
            kind = kind,
            payload = %payload,
            "contract_event"
        ),
        EventLevel::Warn => tracing::warn!(
            target: "observability.contract",
            subsystem = subsystem,
            family = family,
            run_id = %run_id,
            tick = tick,
            tick_present = tick_present,
            organ_id = organ_id,
            thread_id = thread_id,
            turn_id = turn_id,
            request_id = request_id,
            endpoint_id = endpoint_id,
            descriptor_id = descriptor_id,
            act_id = act_id,
            sense_id = sense_id,
            adapter_id = adapter_id,
            adapter_type = adapter_type,
            transition_kind = transition_kind,
            outcome = outcome,
            direction = direction,
            binding_kind = binding_kind,
            change_mode = change_mode,
            state = state,
            kind = kind,
            payload = %payload,
            "contract_event"
        ),
    }
}
