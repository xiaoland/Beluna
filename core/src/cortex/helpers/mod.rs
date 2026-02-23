use async_trait::async_trait;
use serde::Serialize;

use crate::{
    ai_gateway::types_chat::{ChatResponse, OutputMode},
    cortex::{error::CortexError, testing::TestHooks, types::ReactionLimits},
    types::{NeuralSignalDescriptor, NeuralSignalType},
};

pub(crate) mod act_descriptor_input_helper;
pub(crate) mod acts_output_helper;
pub(crate) mod goal_tree_input_helper;
pub(crate) mod goal_tree_patch_output_helper;
pub(crate) mod l1_memory_flush_output_helper;
pub(crate) mod sense_input_helper;

#[derive(Clone, Copy)]
pub(crate) enum CognitionOrgan {
    Primary,
    Sense,
    ActDescriptor,
    GoalTree,
    Acts,
    GoalTreePatch,
    L1MemoryFlush,
}

impl CognitionOrgan {
    pub(crate) fn stage(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Sense => "sense_helper",
            Self::ActDescriptor => "act_descriptor_helper",
            Self::GoalTree => "goal_tree_helper",
            Self::Acts => "acts_helper",
            Self::GoalTreePatch => "goal_tree_patch_helper",
            Self::L1MemoryFlush => "l1_memory_flush_helper",
        }
    }
}

#[async_trait]
pub(crate) trait HelperRuntime: Send + Sync {
    fn limits(&self) -> &ReactionLimits;
    fn hooks(&self) -> Option<&TestHooks>;

    async fn run_text_organ_with_system(
        &self,
        cycle_id: u64,
        organ: CognitionOrgan,
        max_output_tokens: u64,
        system_prompt: String,
        user_prompt: String,
    ) -> Result<String, CortexError>;

    async fn run_organ(
        &self,
        cycle_id: u64,
        organ: CognitionOrgan,
        max_output_tokens: u64,
        system_prompt: String,
        user_prompt: String,
        output_mode: OutputMode,
    ) -> Result<ChatResponse, CortexError>;
}

#[derive(Clone, Default)]
pub(crate) struct InputHelpers {
    pub sense: sense_input_helper::SenseInputHelper,
    pub act_descriptor: act_descriptor_input_helper::ActDescriptorInputHelper,
    pub goal_tree: goal_tree_input_helper::GoalTreeInputHelper,
}

#[derive(Clone, Default)]
pub(crate) struct OutputHelpers {
    pub acts: acts_output_helper::ActsOutputHelper,
    pub goal_tree_patch: goal_tree_patch_output_helper::GoalTreePatchOutputHelper,
    pub l1_memory_flush: l1_memory_flush_output_helper::L1MemoryFlushOutputHelper,
}

#[derive(Clone, Default)]
pub(crate) struct CortexHelpers {
    pub input: InputHelpers,
    pub output: OutputHelpers,
}

pub(crate) fn log_organ_input(cycle_id: u64, stage: &str, input_payload: &str) {
    tracing::info!(
        target: "cortex",
        cycle_id = cycle_id,
        stage = stage,
        input_payload = %input_payload,
        "cortex_organ_input"
    );
}

pub(crate) fn log_organ_output(cycle_id: u64, stage: &str, output_payload: &str) {
    tracing::info!(
        target: "cortex",
        cycle_id = cycle_id,
        stage = stage,
        output_payload = %output_payload,
        "cortex_organ_output"
    );
}

pub(crate) fn pretty_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value)
        .unwrap_or_else(|err| format!("{{\"serialization_error\":\"{}\"}}", err))
}

pub(crate) fn sense_descriptors(
    capability_entries: &[NeuralSignalDescriptor],
) -> Vec<NeuralSignalDescriptor> {
    let mut entries: Vec<_> = capability_entries
        .iter()
        .filter(|entry| entry.r#type == NeuralSignalType::Sense)
        .cloned()
        .collect();
    entries.sort_by(|lhs, rhs| {
        lhs.endpoint_id.cmp(&rhs.endpoint_id).then_with(|| {
            lhs.neural_signal_descriptor_id
                .cmp(&rhs.neural_signal_descriptor_id)
        })
    });
    entries.dedup_by(|lhs, rhs| {
        lhs.endpoint_id == rhs.endpoint_id
            && lhs.neural_signal_descriptor_id == rhs.neural_signal_descriptor_id
            && lhs.payload_schema == rhs.payload_schema
    });
    entries
}

pub(crate) fn act_descriptors(
    capability_entries: &[NeuralSignalDescriptor],
) -> Vec<NeuralSignalDescriptor> {
    let mut entries: Vec<_> = capability_entries
        .iter()
        .filter(|entry| entry.r#type == NeuralSignalType::Act)
        .cloned()
        .collect();
    entries.sort_by(|lhs, rhs| {
        lhs.endpoint_id.cmp(&rhs.endpoint_id).then_with(|| {
            lhs.neural_signal_descriptor_id
                .cmp(&rhs.neural_signal_descriptor_id)
        })
    });
    entries.dedup_by(|lhs, rhs| {
        lhs.endpoint_id == rhs.endpoint_id
            && lhs.neural_signal_descriptor_id == rhs.neural_signal_descriptor_id
            && lhs.payload_schema == rhs.payload_schema
    });
    entries
}
