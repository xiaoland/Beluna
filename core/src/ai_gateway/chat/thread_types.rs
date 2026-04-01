use std::{collections::BTreeMap, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::ai_gateway::types::ChatRouteRef;

use super::{
    executor::ToolExecutor,
    tool::{ChatToolDefinition, ToolOverride},
    turn::Turn,
    types::{ChatMessage, OutputMode, TurnLimits, TurnResponse},
};

#[derive(Debug, Clone, Default)]
pub struct TurnQuery {
    pub min_turn_id: Option<u64>,
    pub max_turn_id: Option<u64>,
    pub has_tool_calls: Option<bool>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct TurnRef {
    pub turn_id: u64,
    pub index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnSummary {
    pub turn_id: u64,
    pub message_count: usize,
    pub tool_call_count: usize,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TurnRetentionPolicy {
    KeepAll,
    KeepLastTurns { count: usize },
    KeepSelectedTurnIds { turn_ids: Vec<u64> },
    DropAll,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SystemPromptAction {
    Keep,
    Clear,
    Replace { prompt: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContextControlReason {
    Manual,
    CortexReset,
    ContinuationCleanup,
    Recovery,
}

impl ContextControlReason {
    pub fn as_label(&self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::CortexReset => "cortex_reset",
            Self::ContinuationCleanup => "continuation_cleanup",
            Self::Recovery => "recovery",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThreadContextRequest {
    pub retention: TurnRetentionPolicy,
    pub system_prompt: SystemPromptAction,
    pub drop_unfinished_continuation: bool,
    pub reason: ContextControlReason,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThreadContextResult {
    pub kept_turn_ids: Vec<u64>,
    pub dropped_turn_ids: Vec<u64>,
    pub continuation_dropped: bool,
}

#[derive(Default)]
pub struct ThreadOptions {
    pub thread_id: Option<String>,
    pub route_ref: Option<ChatRouteRef>,
    pub tools: Vec<ChatToolDefinition>,
    pub system_prompt: Option<String>,
    pub default_output_mode: Option<OutputMode>,
    pub default_limits: Option<TurnLimits>,
    pub enable_thinking: bool,
    pub seed_turns: Vec<Turn>,
    pub metadata: BTreeMap<String, String>,
}

pub struct DeriveContextOptions {
    pub thread_id: Option<String>,
    pub route_ref: Option<ChatRouteRef>,
    pub metadata: BTreeMap<String, String>,
}

impl Default for DeriveContextOptions {
    fn default() -> Self {
        Self {
            thread_id: None,
            route_ref: None,
            metadata: BTreeMap::new(),
        }
    }
}

#[derive(Default)]
pub struct RewriteContextOptions {
    pub metadata: BTreeMap<String, String>,
}

pub struct TurnInput {
    pub messages: Vec<ChatMessage>,
    pub tool_overrides: Vec<ToolOverride>,
    pub output_mode: Option<OutputMode>,
    pub limits: Option<TurnLimits>,
    pub enable_thinking: Option<bool>,
    pub tool_executor: Option<Arc<dyn ToolExecutor>>,
    pub metadata: BTreeMap<String, String>,
}

impl Default for TurnInput {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            tool_overrides: Vec::new(),
            output_mode: None,
            limits: None,
            enable_thinking: None,
            tool_executor: None,
            metadata: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TurnOutput {
    pub chat_id: String,
    pub thread_id: String,
    pub turn_id: u64,
    pub response: TurnResponse,
}
