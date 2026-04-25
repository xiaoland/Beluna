use serde_json::{Value, json};

use crate::{
    cortex::GoalNode,
    observability::contract::{
        ContractEvent, CortexGoalForestEvent, CortexOrganExecutionEvent, OrganResponseStatus,
    },
};

use super::{current_run_id, emit_contract_event, timestamp_now};

pub fn emit_cortex_organ_start(
    tick: u64,
    organ_id: &str,
    route_or_backend: Option<&str>,
    request_id: &str,
    input_payload: Value,
) {
    emit_contract_event(contract_event_for_organ(
        organ_id,
        CortexOrganExecutionEvent {
            run_id: current_run_id().to_string(),
            timestamp: timestamp_now(),
            tick,
            request_id: request_id.to_string(),
            span_id: request_id.to_string(),
            parent_span_id: None,
            phase: "start".to_string(),
            status: OrganResponseStatus::Ok,
            route_or_backend: route_or_backend.map(str::to_string),
            input_payload: Some(input_payload),
            output_payload: None,
            error: None,
            ai_request_id: None,
            thread_id: None,
            turn_id: None,
        },
    ));
}

pub fn emit_cortex_organ_end(
    tick: u64,
    organ_id: &str,
    request_id: &str,
    status: OrganResponseStatus,
    output_payload: Option<Value>,
    error: Option<Value>,
    ai_request_id: Option<&str>,
    thread_id: Option<&str>,
    turn_id: Option<u64>,
) {
    emit_contract_event(contract_event_for_organ(
        organ_id,
        CortexOrganExecutionEvent {
            run_id: current_run_id().to_string(),
            timestamp: timestamp_now(),
            tick,
            request_id: request_id.to_string(),
            span_id: request_id.to_string(),
            parent_span_id: None,
            phase: "end".to_string(),
            status,
            route_or_backend: None,
            input_payload: None,
            output_payload,
            error,
            ai_request_id: ai_request_id.map(str::to_string),
            thread_id: thread_id.map(str::to_string),
            turn_id,
        },
    ));
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
        parent_span_id: None,
        kind: "snapshot".to_string(),
        snapshot: Some(snapshot.clone()),
        mutation_request: None,
        mutation_result: None,
        persisted_revision: None,
        reset_context_applied: None,
        selected_turn_ids: None,
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
        parent_span_id: None,
        kind: "mutation".to_string(),
        snapshot: None,
        mutation_request: patch_request_when_present,
        mutation_result: patch_result_when_present,
        persisted_revision: cognition_persisted_revision_when_present,
        reset_context_applied: reset_context_applied_when_present,
        selected_turn_ids: selected_turn_ids_when_present,
    }));
}

fn contract_event_for_organ(organ_id: &str, event: CortexOrganExecutionEvent) -> ContractEvent {
    match organ_id {
        "primary" => ContractEvent::CortexPrimary(event),
        "attention" => ContractEvent::CortexAttention(event),
        "cleanup" => ContractEvent::CortexCleanup(event),
        "sense_helper" => ContractEvent::CortexSenseHelper(event),
        "goal_forest_helper" => ContractEvent::CortexGoalForestHelper(event),
        "acts_helper" => ContractEvent::CortexActsHelper(event),
        other => {
            tracing::warn!(
                target: "observability.contract",
                organ_id = other,
                "unknown_cortex_organ_family_falling_back_to_primary"
            );
            ContractEvent::CortexPrimary(event)
        }
    }
}

fn count_goal_nodes(goal_forest_nodes: &[GoalNode]) -> usize {
    goal_forest_nodes
        .iter()
        .map(|node| 1 + count_goal_nodes(&node.children))
        .sum()
}
