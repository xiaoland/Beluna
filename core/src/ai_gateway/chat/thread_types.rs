use std::{collections::BTreeMap, sync::Arc};

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

#[derive(Debug, Clone)]
pub struct TurnSummary {
    pub turn_id: u64,
    pub message_count: usize,
    pub tool_call_count: usize,
    pub completed: bool,
}

#[derive(Default)]
pub struct ThreadOptions {
    pub thread_id: Option<String>,
    pub route_or_alias: Option<String>,
    pub tools: Vec<ChatToolDefinition>,
    pub system_prompt: Option<String>,
    pub default_output_mode: Option<OutputMode>,
    pub default_limits: Option<TurnLimits>,
    pub enable_thinking: bool,
    pub seed_turns: Vec<Turn>,
}

pub struct CloneThreadOptions {
    pub thread_id: Option<String>,
    pub route_or_alias: Option<String>,
    pub system_prompt: Option<String>,
}

impl Default for CloneThreadOptions {
    fn default() -> Self {
        Self {
            thread_id: None,
            route_or_alias: None,
            system_prompt: None,
        }
    }
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
