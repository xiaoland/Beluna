use serde_json::{Value, json};

use crate::{cortex::GoalNode, observability::owner_log};

use super::{OrganResponseStatus, current_run_id};

pub fn emit_cortex_organ_start(
    tick: u64,
    organ_id: &str,
    route_or_backend: Option<&str>,
    _request_id: &str,
    input_payload: Value,
) {
    if organ_id == "primary" {
        owner_log::events::emit_primary_started(route_or_backend, tick, input_payload.clone());
    }
}

pub fn emit_cortex_organ_end(
    tick: u64,
    organ_id: &str,
    _request_id: &str,
    status: OrganResponseStatus,
    output_payload: Option<Value>,
    error: Option<Value>,
    ai_request_id: Option<&str>,
    thread_id: Option<&str>,
    turn_id: Option<u64>,
) {
    if organ_id == "primary" {
        owner_log::events::emit_primary_finished(
            tick,
            status,
            output_payload.clone(),
            error.clone(),
            ai_request_id,
            thread_id,
            turn_id,
        );
    }
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

    snapshot
}

pub fn emit_cortex_goal_forest_patch(
    _tick: u64,
    _span_id: &str,
    _patch_request_when_present: Option<Value>,
    _patch_result_when_present: Option<Value>,
    _cognition_persisted_revision_when_present: Option<u64>,
    _reset_context_applied_when_present: Option<bool>,
    _selected_turn_ids_when_present: Option<Vec<u64>>,
) {
}

fn count_goal_nodes(goal_forest_nodes: &[GoalNode]) -> usize {
    goal_forest_nodes
        .iter()
        .map(|node| 1 + count_goal_nodes(&node.children))
        .sum()
}
