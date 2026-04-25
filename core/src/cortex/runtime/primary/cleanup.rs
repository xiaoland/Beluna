use std::{collections::BTreeSet, sync::Arc};

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::{
    ai_gateway::chat::{
        ContextControlReason, Thread, ToolCallResult, ToolExecutionRequest, ToolExecutionResult,
        ToolExecutor, ToolOverride,
    },
    cortex::{
        error::{CortexError, extractor_failed},
        helpers::{self, CognitionOrgan, goal_forest_helper::GoalNode},
        prompts,
    },
};

use super::{
    Cortex,
    tools::{PRIMARY_TOOL_BREAK_PRIMARY_PHASE, PRIMARY_TOOL_EXPAND_SENSES},
};

const CLEANUP_TOOL_PATCH_GOAL_FOREST: &str = "patch-goal-forest";
const CLEANUP_TOOL_RESET_CONTEXT: &str = "reset-context";

#[derive(Debug, Clone, Default, serde::Serialize)]
pub(super) struct CleanupPhaseOutput {
    pub(super) patched_goal_forest: Option<Vec<GoalNode>>,
    pub(super) reset_context_requested: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "op", rename_all = "kebab-case")]
enum GoalForestPatchOperation {
    AddRoot {
        node: GoalNode,
    },
    ReplaceNode {
        node_id: String,
        node: GoalNode,
    },
    RemoveNode {
        node_id: String,
    },
    InsertChild {
        parent_id: String,
        #[serde(default)]
        index: Option<usize>,
        node: GoalNode,
    },
    ReplaceChildren {
        parent_id: String,
        #[serde(default)]
        children: Vec<GoalNode>,
    },
    UpdateFields {
        node_id: String,
        #[serde(default)]
        status: Option<String>,
        #[serde(default)]
        weight: Option<f64>,
        #[serde(default)]
        summary: Option<String>,
    },
}

#[derive(Debug, serde::Deserialize)]
struct PatchGoalForestArgs {
    #[serde(default)]
    operations: Vec<GoalForestPatchOperation>,
}

#[derive(Debug, Default)]
struct CleanupToolState {
    output: CleanupPhaseOutput,
    patch_called: bool,
    reset_called: bool,
    protocol_violation: Option<String>,
}

#[derive(Clone)]
struct CleanupToolExecutor {
    cycle_id: u64,
    current_goal_forest: Vec<GoalNode>,
    state: Arc<Mutex<CleanupToolState>>,
}

impl CleanupToolExecutor {
    fn new(cycle_id: u64, current_goal_forest: Vec<GoalNode>) -> Self {
        Self {
            cycle_id,
            current_goal_forest,
            state: Arc::new(Mutex::new(CleanupToolState::default())),
        }
    }

    async fn output(&self) -> Result<CleanupPhaseOutput, CortexError> {
        let state = self.state.lock().await;
        if let Some(message) = state.protocol_violation.as_ref() {
            return Err(extractor_failed(message.clone()));
        }
        Ok(state.output.clone())
    }

    async fn execute_cleanup_tool_call(&self, call: &ToolCallResult) -> serde_json::Value {
        let result = match call.name.as_str() {
            CLEANUP_TOOL_PATCH_GOAL_FOREST => {
                let parsed = serde_json::from_str::<PatchGoalForestArgs>(&call.arguments_json)
                    .map_err(|err| err.to_string());
                match parsed {
                    Ok(args) => {
                        let mut state = self.state.lock().await;
                        if state.patch_called {
                            fail_closed(&mut state, "patch-goal-forest was called more than once")
                        } else {
                            match reduce_goal_forest(&self.current_goal_forest, &args.operations) {
                                Ok(nodes) => {
                                    state.patch_called = true;
                                    state.output.patched_goal_forest = Some(nodes);
                                    Ok(serde_json::json!({
                                        "operation_count": args.operations.len(),
                                        "goal_forest": state.output.patched_goal_forest
                                    }))
                                }
                                Err(err) => fail_closed(&mut state, err),
                            }
                        }
                    }
                    Err(err) => {
                        let mut state = self.state.lock().await;
                        fail_closed(&mut state, err)
                    }
                }
            }
            CLEANUP_TOOL_RESET_CONTEXT => {
                let parsed = serde_json::from_str::<serde_json::Value>(&call.arguments_json)
                    .map_err(|err| err.to_string());
                match parsed {
                    Ok(serde_json::Value::Object(map)) if map.is_empty() => {
                        let mut state = self.state.lock().await;
                        if state.reset_called {
                            fail_closed(&mut state, "reset-context was called more than once")
                        } else {
                            state.reset_called = true;
                            state.output.reset_context_requested = true;
                            Ok(serde_json::json!({ "reset_context": "requested" }))
                        }
                    }
                    Ok(_) => {
                        let mut state = self.state.lock().await;
                        fail_closed(&mut state, "reset-context expects an empty object")
                    }
                    Err(err) => {
                        let mut state = self.state.lock().await;
                        fail_closed(&mut state, err)
                    }
                }
            }
            _ => Err(format!("unknown cleanup tool '{}'", call.name)),
        };

        match result {
            Ok(data) => serde_json::json!({
                "ok": true,
                "tool": call.name,
                "data": data,
            }),
            Err(error) => {
                tracing::warn!(
                    target: "cortex",
                    cycle_id = self.cycle_id,
                    tool_name = %call.name,
                    tool_call_id = %call.id,
                    error = %error,
                    "cleanup_tool_failed"
                );
                serde_json::json!({
                    "ok": false,
                    "tool": call.name,
                    "error": error,
                })
            }
        }
    }
}

#[async_trait]
impl ToolExecutor for CleanupToolExecutor {
    async fn execute_call(
        &self,
        request: ToolExecutionRequest,
    ) -> Result<ToolExecutionResult, crate::ai_gateway::error::GatewayError> {
        Ok(ToolExecutionResult {
            payload: self.execute_cleanup_tool_call(&request.call).await,
            reset_messages_applied: false,
        })
    }
}

impl Cortex {
    pub(super) async fn run_cleanup_phase(
        &self,
        cycle_id: u64,
        source_thread: &Thread,
        current_goal_forest: &[GoalNode],
    ) -> Result<CleanupPhaseOutput, CortexError> {
        let organ = CognitionOrgan::Cleanup;
        let stage = organ.stage();
        let request_id = format!("cortex-{stage}-{cycle_id}");
        let thread = self
            .derive_phase_thread(
                cycle_id,
                organ,
                source_thread,
                prompts::cleanup_system_prompt(),
                ContextControlReason::CleanupPhase,
                &request_id,
            )
            .await?;
        let tool_executor = Arc::new(CleanupToolExecutor::new(
            cycle_id,
            current_goal_forest.to_vec(),
        ));
        let response = self
            .run_phase_tool_turn(
                cycle_id,
                organ,
                request_id,
                &thread,
                prompts::cleanup_user_prompt(),
                cleanup_tool_overrides(),
                Some(tool_executor.clone()),
            )
            .await?;
        let output = tool_executor.output().await?;
        helpers::log_organ_output(
            cycle_id,
            stage,
            &helpers::pretty_json(&serde_json::json!({
                "response_text": response.output_text.trim(),
                "output": output,
            })),
        );
        Ok(output)
    }
}

fn reduce_goal_forest(
    current_goal_forest: &[GoalNode],
    operations: &[GoalForestPatchOperation],
) -> Result<Vec<GoalNode>, String> {
    let mut nodes = current_goal_forest.to_vec();
    validate_goal_forest(&nodes)?;
    for operation in operations {
        apply_goal_forest_operation(&mut nodes, operation)?;
        validate_goal_forest(&nodes)?;
    }
    Ok(nodes)
}

fn apply_goal_forest_operation(
    nodes: &mut Vec<GoalNode>,
    operation: &GoalForestPatchOperation,
) -> Result<(), String> {
    match operation {
        GoalForestPatchOperation::AddRoot { node } => {
            nodes.push(node.clone());
            Ok(())
        }
        GoalForestPatchOperation::ReplaceNode { node_id, node } => {
            ensure_target_id(node_id)?;
            replace_node_by_id(nodes, node_id, node.clone())
                .then_some(())
                .ok_or_else(|| format!("node_id '{node_id}' was not found"))
        }
        GoalForestPatchOperation::RemoveNode { node_id } => {
            ensure_target_id(node_id)?;
            remove_node_by_id(nodes, node_id)
                .then_some(())
                .ok_or_else(|| format!("node_id '{node_id}' was not found"))
        }
        GoalForestPatchOperation::InsertChild {
            parent_id,
            index,
            node,
        } => {
            ensure_target_id(parent_id)?;
            let parent = find_node_mut(nodes, parent_id)
                .ok_or_else(|| format!("parent_id '{parent_id}' was not found"))?;
            match index {
                Some(index) if *index > parent.children.len() => Err(format!(
                    "index {index} is out of bounds for parent_id '{parent_id}'"
                )),
                Some(index) => {
                    parent.children.insert(*index, node.clone());
                    Ok(())
                }
                None => {
                    parent.children.push(node.clone());
                    Ok(())
                }
            }
        }
        GoalForestPatchOperation::ReplaceChildren {
            parent_id,
            children,
        } => {
            ensure_target_id(parent_id)?;
            let parent = find_node_mut(nodes, parent_id)
                .ok_or_else(|| format!("parent_id '{parent_id}' was not found"))?;
            parent.children = children.clone();
            Ok(())
        }
        GoalForestPatchOperation::UpdateFields {
            node_id,
            status,
            weight,
            summary,
        } => {
            ensure_target_id(node_id)?;
            let node = find_node_mut(nodes, node_id)
                .ok_or_else(|| format!("node_id '{node_id}' was not found"))?;
            if let Some(status) = status {
                node.status = status.clone();
            }
            if let Some(weight) = weight {
                node.weight = *weight;
            }
            if let Some(summary) = summary {
                node.summary = summary.clone();
            }
            Ok(())
        }
    }
}

fn validate_goal_forest(nodes: &[GoalNode]) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    for node in nodes {
        validate_goal_node(node, &mut ids)?;
    }
    Ok(())
}

fn validate_goal_node(node: &GoalNode, ids: &mut BTreeSet<String>) -> Result<(), String> {
    if node.id.trim().is_empty() {
        return Err("goal node id cannot be empty".to_string());
    }
    if node.status.trim().is_empty() {
        return Err(format!("goal node '{}' status cannot be empty", node.id));
    }
    if node.summary.trim().is_empty() {
        return Err(format!("goal node '{}' summary cannot be empty", node.id));
    }
    if !node.weight.is_finite() || !(0.0..=1.0).contains(&node.weight) {
        return Err(format!("goal node '{}' weight must be in [0,1]", node.id));
    }
    if !ids.insert(node.id.clone()) {
        return Err(format!("duplicate goal node id '{}'", node.id));
    }
    for child in &node.children {
        validate_goal_node(child, ids)?;
    }
    Ok(())
}

fn ensure_target_id(value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err("target node id cannot be empty".to_string());
    }
    Ok(())
}

fn find_node_mut<'a>(nodes: &'a mut [GoalNode], node_id: &str) -> Option<&'a mut GoalNode> {
    for node in nodes {
        if node.id == node_id {
            return Some(node);
        }
        if let Some(found) = find_node_mut(&mut node.children, node_id) {
            return Some(found);
        }
    }
    None
}

fn replace_node_by_id(nodes: &mut [GoalNode], node_id: &str, replacement: GoalNode) -> bool {
    for node in nodes {
        if node.id == node_id {
            *node = replacement;
            return true;
        }
        if replace_node_by_id(&mut node.children, node_id, replacement.clone()) {
            return true;
        }
    }
    false
}

fn remove_node_by_id(nodes: &mut Vec<GoalNode>, node_id: &str) -> bool {
    if let Some(index) = nodes.iter().position(|node| node.id == node_id) {
        nodes.remove(index);
        return true;
    }
    for node in nodes {
        if remove_node_by_id(&mut node.children, node_id) {
            return true;
        }
    }
    false
}

fn fail_closed<T>(state: &mut CleanupToolState, message: impl Into<String>) -> Result<T, String> {
    let message = message.into();
    state.protocol_violation = Some(message.clone());
    Err(message)
}

fn cleanup_tool_overrides() -> Vec<ToolOverride> {
    vec![
        ToolOverride::Remove(PRIMARY_TOOL_EXPAND_SENSES.to_string()),
        ToolOverride::Remove(PRIMARY_TOOL_BREAK_PRIMARY_PHASE.to_string()),
        ToolOverride::Set(crate::ai_gateway::chat::ChatToolDefinition {
            name: CLEANUP_TOOL_PATCH_GOAL_FOREST.to_string(),
            description: Some(
                concat!(
                    "Patch the goal forest using deterministic operations. ",
                    "The runtime reducer applies operations in order and rejects invalid sequences."
                )
                .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "operations": {
                        "type": "array",
                        "items": goal_forest_operation_schema()
                    }
                },
                "required": ["operations"],
                "additionalProperties": false
            }),
        }),
        ToolOverride::Set(crate::ai_gateway::chat::ChatToolDefinition {
            name: CLEANUP_TOOL_RESET_CONTEXT.to_string(),
            description: Some(
                "Clear Primary thread history after this tick, keeping only the Primary system prompt."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        }),
    ]
}

fn goal_forest_operation_schema() -> serde_json::Value {
    let goal_node_schema = serde_json::json!({
        "type": "object",
        "properties": {
            "status": { "type": "string", "minLength": 1 },
            "weight": { "type": "number", "minimum": 0, "maximum": 1 },
            "id": { "type": "string", "minLength": 1 },
            "summary": { "type": "string", "minLength": 1 },
            "children": {
                "type": "array",
                "items": {}
            }
        },
        "required": ["status", "weight", "id", "summary", "children"],
        "additionalProperties": false
    });
    serde_json::json!({
        "oneOf": [
            {
                "type": "object",
                "properties": {
                    "op": { "type": "string", "enum": ["add-root"] },
                    "node": goal_node_schema.clone()
                },
                "required": ["op", "node"],
                "additionalProperties": false
            },
            {
                "type": "object",
                "properties": {
                    "op": { "type": "string", "enum": ["replace-node"] },
                    "node_id": { "type": "string", "minLength": 1 },
                    "node": goal_node_schema.clone()
                },
                "required": ["op", "node_id", "node"],
                "additionalProperties": false
            },
            {
                "type": "object",
                "properties": {
                    "op": { "type": "string", "enum": ["remove-node"] },
                    "node_id": { "type": "string", "minLength": 1 }
                },
                "required": ["op", "node_id"],
                "additionalProperties": false
            },
            {
                "type": "object",
                "properties": {
                    "op": { "type": "string", "enum": ["insert-child"] },
                    "parent_id": { "type": "string", "minLength": 1 },
                    "index": { "type": "integer", "minimum": 0 },
                    "node": goal_node_schema.clone()
                },
                "required": ["op", "parent_id", "node"],
                "additionalProperties": false
            },
            {
                "type": "object",
                "properties": {
                    "op": { "type": "string", "enum": ["replace-children"] },
                    "parent_id": { "type": "string", "minLength": 1 },
                    "children": {
                        "type": "array",
                        "items": goal_node_schema.clone()
                    }
                },
                "required": ["op", "parent_id", "children"],
                "additionalProperties": false
            },
            {
                "type": "object",
                "properties": {
                    "op": { "type": "string", "enum": ["update-fields"] },
                    "node_id": { "type": "string", "minLength": 1 },
                    "status": { "type": "string", "minLength": 1 },
                    "weight": { "type": "number", "minimum": 0, "maximum": 1 },
                    "summary": { "type": "string", "minLength": 1 }
                },
                "required": ["op", "node_id"],
                "additionalProperties": false
            }
        ]
    })
}
