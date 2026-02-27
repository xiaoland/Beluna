use async_trait::async_trait;
use serde::Serialize;

use crate::{
    ai_gateway::chat::{OutputMode, TurnResponse},
    cortex::{error::CortexError, testing::TestHooks, types::ReactionLimits},
    types::{NeuralSignalDescriptor, NeuralSignalType},
};

pub(crate) mod act_descriptor_input_helper;
pub(crate) mod acts_output_helper;
pub(crate) mod goal_forest_helper;
pub(crate) mod l1_memory_flush_output_helper;
pub(crate) mod l1_memory_input_helper;
pub(crate) mod proprioception_input_helper;
pub(crate) mod sense_input_helper;

#[derive(Clone, Copy)]
pub(crate) enum CognitionOrgan {
    Primary,
    Sense,
    ActDescriptor,
    GoalForest,
    Acts,
    L1MemoryFlush,
}

impl CognitionOrgan {
    pub(crate) fn stage(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Sense => "sense_helper",
            Self::ActDescriptor => "act_descriptor_helper",
            Self::GoalForest => "goal_forest_helper",
            Self::Acts => "acts_helper",
            Self::L1MemoryFlush => "l1_memory_flush_helper",
        }
    }
}

#[async_trait]
pub(crate) trait HelperRuntime: Send + Sync {
    fn limits(&self) -> &ReactionLimits;
    fn hooks(&self) -> Option<&TestHooks>;
    fn emit_stage_failed(&self, cycle_id: u64, stage: &'static str);

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
    ) -> Result<TurnResponse, CortexError>;
}

#[derive(Clone, Default)]
pub(crate) struct InputHelper {
    pub sense: sense_input_helper::SenseInputHelper,
    pub proprioception: proprioception_input_helper::ProprioceptionInputHelper,
    pub act_descriptor: act_descriptor_input_helper::ActDescriptorInputHelper,
    pub goal_forest: goal_forest_helper::GoalForestHelper,
    pub l1_memory: l1_memory_input_helper::L1MemoryInputHelper,
}

#[derive(Clone, Default)]
pub(crate) struct OutputHelper {
    pub acts: acts_output_helper::ActsOutputHelper,
    pub l1_memory_flush: l1_memory_flush_output_helper::L1MemoryFlushOutputHelper,
}

#[derive(Clone, Default)]
pub(crate) struct CortexHelper {
    pub input: InputHelper,
    pub output: OutputHelper,
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
