use serde_json::{Value, json};

use crate::{
    cortex::GoalNode,
    observability::contract::{
        ContractEvent, CortexGoalForestSnapshotEvent, CortexOrganRequestEvent,
        CortexOrganResponseEvent, CortexTickEvent, OrganResponseStatus,
    },
    types::{PhysicalState, Sense},
};

use super::{current_run_id, emit_contract_event, timestamp_now};

pub fn emit_cortex_tick(
    tick: u64,
    trigger_summary: Value,
    senses: &[Sense],
    physical_state: &PhysicalState,
    dispatched_act_count: usize,
    goal_forest_snapshot_id: String,
) {
    emit_contract_event(ContractEvent::CortexTick(CortexTickEvent {
        run_id: current_run_id().to_string(),
        timestamp: timestamp_now(),
        tick,
        trigger_summary,
        senses_summary: senses_summary(senses),
        proprioception_snapshot_or_ref: json!({
            "kind": "inline_snapshot",
            "entries": &physical_state.proprioception,
        }),
        acts_summary: json!({
            "dispatched_act_count": dispatched_act_count,
        }),
        goal_forest_ref: json!({
            "kind": "snapshot_id",
            "value": goal_forest_snapshot_id,
        }),
    }));
}

pub fn emit_cortex_organ_request(
    tick: u64,
    stage: &str,
    route_or_organ: Option<&str>,
    request_id: &str,
    input_summary: Value,
) {
    emit_contract_event(ContractEvent::CortexOrganRequest(CortexOrganRequestEvent {
        run_id: current_run_id().to_string(),
        timestamp: timestamp_now(),
        tick,
        stage: stage.to_string(),
        route_or_organ: route_or_organ.unwrap_or(stage).to_string(),
        request_id: request_id.to_string(),
        input_summary,
    }));
}

pub fn emit_cortex_organ_response(
    tick: u64,
    stage: &str,
    request_id: &str,
    status: OrganResponseStatus,
    response_summary: Value,
    tool_summary: Value,
    act_summary: Value,
    error_summary_when_present: Option<Value>,
) {
    emit_contract_event(ContractEvent::CortexOrganResponse(
        CortexOrganResponseEvent {
            run_id: current_run_id().to_string(),
            timestamp: timestamp_now(),
            tick,
            stage: stage.to_string(),
            request_id: request_id.to_string(),
            status,
            response_summary,
            tool_summary,
            act_summary,
            error_summary_when_present,
        },
    ));
}

pub fn emit_cortex_goal_forest_snapshot(tick: u64, goal_forest_nodes: &[GoalNode]) -> String {
    let snapshot_id = format!("goal-forest:{}:{tick}", current_run_id());
    let snapshot_nodes = serde_json::to_value(goal_forest_nodes).unwrap_or_else(|_| json!([]));

    emit_contract_event(ContractEvent::CortexGoalForestSnapshot(
        CortexGoalForestSnapshotEvent {
            run_id: current_run_id().to_string(),
            timestamp: timestamp_now(),
            tick,
            snapshot_summary: json!({
                "root_count": goal_forest_nodes.len(),
                "total_goal_count": count_goal_nodes(goal_forest_nodes),
            }),
            snapshot_or_ref: json!({
                "kind": "inline_snapshot",
                "snapshot_id": snapshot_id,
                "nodes": snapshot_nodes,
            }),
        },
    ));

    snapshot_id
}

fn senses_summary(senses: &[Sense]) -> Value {
    serde_json::to_value(
        senses
            .iter()
            .map(|sense| {
                json!({
                    "endpoint_id": &sense.endpoint_id,
                    "descriptor_id": &sense.neural_signal_descriptor_id,
                    "sense_id": &sense.sense_instance_id,
                    "act_id_when_present": sense.act_instance_id.as_deref(),
                })
            })
            .collect::<Vec<_>>(),
    )
    .unwrap_or_else(|_| json!([]))
}

fn count_goal_nodes(goal_forest_nodes: &[GoalNode]) -> usize {
    goal_forest_nodes
        .iter()
        .map(|node| 1 + count_goal_nodes(&node.children))
        .sum()
}
