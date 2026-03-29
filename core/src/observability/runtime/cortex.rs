use serde_json::{Value, json};

use crate::{
    cortex::GoalNode,
    observability::contract::{
        ContractEvent, CortexGoalForestEvent, CortexOrganEvent, CortexTickEvent,
        OrganResponseStatus,
    },
    types::{PhysicalState, Sense},
};

use super::{current_run_id, emit_contract_event, timestamp_now};

pub fn emit_cortex_tick(
    tick: u64,
    kind_or_status: &str,
    tick_seq_when_present: Option<u64>,
    senses: &[Sense],
    physical_state: &PhysicalState,
    acts_payload_or_summary_when_present: Option<Value>,
    goal_forest_snapshot_ref_or_payload_when_present: Option<Value>,
    error_when_present: Option<Value>,
) {
    emit_contract_event(ContractEvent::CortexTick(CortexTickEvent {
        run_id: current_run_id().to_string(),
        timestamp: timestamp_now(),
        tick,
        span_id: format!("cortex.tick:{tick}"),
        kind_or_status: kind_or_status.to_string(),
        tick_seq_when_present,
        drained_senses: serde_json::to_value(senses).unwrap_or_else(|_| json!([])),
        physical_state_snapshot: serde_json::to_value(physical_state)
            .unwrap_or_else(|_| json!({ "serialization_error": true })),
        control_gate_state_when_present: None,
        acts_payload_or_summary_when_present,
        goal_forest_snapshot_ref_or_payload_when_present,
        error_when_present,
    }));
}

pub fn emit_cortex_organ_start(
    tick: u64,
    organ_id: &str,
    route_or_backend_when_present: Option<&str>,
    request_id: &str,
    input_payload: Value,
) {
    emit_contract_event(ContractEvent::CortexOrgan(CortexOrganEvent {
        run_id: current_run_id().to_string(),
        timestamp: timestamp_now(),
        tick,
        organ_id: organ_id.to_string(),
        request_id: request_id.to_string(),
        span_id: request_id.to_string(),
        parent_span_id_when_present: None,
        route_or_backend_when_present: route_or_backend_when_present.map(str::to_string),
        phase: "start".to_string(),
        status: OrganResponseStatus::Ok,
        input_payload_when_present: Some(input_payload),
        output_payload_when_present: None,
        error_when_present: None,
        ai_gateway_request_id_when_present: None,
        thread_id_when_present: None,
        turn_id_when_present: None,
    }));
}

pub fn emit_cortex_organ_end(
    tick: u64,
    organ_id: &str,
    request_id: &str,
    status: OrganResponseStatus,
    output_payload_when_present: Option<Value>,
    error_when_present: Option<Value>,
    ai_gateway_request_id_when_present: Option<&str>,
    thread_id_when_present: Option<&str>,
    turn_id_when_present: Option<u64>,
) {
    emit_contract_event(ContractEvent::CortexOrgan(CortexOrganEvent {
        run_id: current_run_id().to_string(),
        timestamp: timestamp_now(),
        tick,
        organ_id: organ_id.to_string(),
        request_id: request_id.to_string(),
        span_id: request_id.to_string(),
        parent_span_id_when_present: None,
        route_or_backend_when_present: None,
        phase: "end".to_string(),
        status,
        input_payload_when_present: None,
        output_payload_when_present,
        error_when_present,
        ai_gateway_request_id_when_present: ai_gateway_request_id_when_present.map(str::to_string),
        thread_id_when_present: thread_id_when_present.map(str::to_string),
        turn_id_when_present,
    }));
}

pub fn emit_cortex_goal_forest_snapshot(tick: u64, goal_forest_nodes: &[GoalNode]) -> Value {
    let snapshot_id = format!("goal-forest:{}:{tick}", current_run_id());
    let snapshot = json!({
        "kind": "inline_snapshot",
        "snapshot_id": snapshot_id,
        "nodes": serde_json::to_value(goal_forest_nodes).unwrap_or_else(|_| json!([])),
        "root_count": goal_forest_nodes.len(),
        "total_goal_count": count_goal_nodes(goal_forest_nodes),
    });

    emit_contract_event(ContractEvent::CortexGoalForest(CortexGoalForestEvent {
        run_id: current_run_id().to_string(),
        timestamp: timestamp_now(),
        tick,
        span_id: format!("cortex.goal-forest.snapshot:{tick}"),
        parent_span_id_when_present: None,
        kind: "snapshot".to_string(),
        snapshot_when_present: Some(snapshot.clone()),
        patch_request_when_present: None,
        patch_result_when_present: None,
        cognition_persisted_revision_when_present: None,
        reset_context_applied_when_present: None,
        selected_turn_ids_when_present: None,
    }));

    snapshot
}

pub fn emit_cortex_goal_forest_patch(
    tick: u64,
    span_id: &str,
    patch_request_when_present: Option<Value>,
    patch_result_when_present: Option<Value>,
    cognition_persisted_revision_when_present: Option<u64>,
    reset_context_applied_when_present: Option<bool>,
    selected_turn_ids_when_present: Option<Vec<u64>>,
) {
    emit_contract_event(ContractEvent::CortexGoalForest(CortexGoalForestEvent {
        run_id: current_run_id().to_string(),
        timestamp: timestamp_now(),
        tick,
        span_id: span_id.to_string(),
        parent_span_id_when_present: None,
        kind: "patch".to_string(),
        snapshot_when_present: None,
        patch_request_when_present,
        patch_result_when_present,
        cognition_persisted_revision_when_present,
        reset_context_applied_when_present,
        selected_turn_ids_when_present,
    }));
}

fn count_goal_nodes(goal_forest_nodes: &[GoalNode]) -> usize {
    goal_forest_nodes
        .iter()
        .map(|node| 1 + count_goal_nodes(&node.children))
        .sum()
}
