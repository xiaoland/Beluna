use std::{future::Future, pin::Pin, sync::Arc};

use crate::{
    cortex::{Cortex, CortexError, ReactionLimits},
    types::{CognitionState, NeuralSignalDescriptor, PhysicalState, Sense},
};

#[derive(Debug, Clone)]
pub struct SenseHelperRequest {
    pub cycle_id: u64,
    pub senses: Vec<Sense>,
    pub sense_descriptors: Vec<NeuralSignalDescriptor>,
}

#[derive(Debug, Clone)]
pub struct ActDescriptorHelperRequest {
    pub cycle_id: u64,
    pub act_descriptors: Vec<NeuralSignalDescriptor>,
}

#[derive(Debug, Clone)]
pub struct PrimaryRequest {
    pub cycle_id: u64,
    pub senses: Vec<Sense>,
    pub physical_state: PhysicalState,
    pub cognition_state: CognitionState,
    pub input_ir: String,
}

#[derive(Debug, Clone)]
pub struct ActsHelperRequest {
    pub cycle_id: u64,
    pub output_ir: String,
    pub acts_section: String,
}

#[derive(Debug, Clone)]
pub struct GoalStackHelperRequest {
    pub cycle_id: u64,
    pub output_ir: String,
    pub goal_stack_patch_section: String,
    pub cognition_state: CognitionState,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestActDraft {
    pub endpoint_id: String,
    pub neural_signal_descriptor_id: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct TestActsHelperOutput {
    #[serde(default)]
    pub acts: Vec<TestActDraft>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct TestGoalStackPatch {
    #[serde(default)]
    pub ops: Vec<TestGoalStackPatchOp>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum TestGoalStackPatchOp {
    Push { goal_id: String, summary: String },
    Pop,
    ReplaceTop { goal_id: String, summary: String },
    Clear,
}

type SenseHelperFuture = Pin<Box<dyn Future<Output = Result<String, CortexError>> + Send>>;
type ActDescriptorHelperFuture = Pin<Box<dyn Future<Output = Result<String, CortexError>> + Send>>;
type PrimaryFuture = Pin<Box<dyn Future<Output = Result<String, CortexError>> + Send>>;
type ActsHelperFuture =
    Pin<Box<dyn Future<Output = Result<TestActsHelperOutput, CortexError>> + Send>>;
type GoalStackHelperFuture =
    Pin<Box<dyn Future<Output = Result<TestGoalStackPatch, CortexError>> + Send>>;

pub type SenseHelperHook = Arc<dyn Fn(SenseHelperRequest) -> SenseHelperFuture + Send + Sync>;
pub type ActDescriptorHelperHook =
    Arc<dyn Fn(ActDescriptorHelperRequest) -> ActDescriptorHelperFuture + Send + Sync>;
pub type PrimaryHook = Arc<dyn Fn(PrimaryRequest) -> PrimaryFuture + Send + Sync>;
pub type ActsHelperHook = Arc<dyn Fn(ActsHelperRequest) -> ActsHelperFuture + Send + Sync>;
pub type GoalStackHelperHook =
    Arc<dyn Fn(GoalStackHelperRequest) -> GoalStackHelperFuture + Send + Sync>;

pub fn boxed<T>(
    future: impl Future<Output = T> + Send + 'static,
) -> Pin<Box<dyn Future<Output = T> + Send>>
where
    T: Send + 'static,
{
    Box::pin(future)
}

#[derive(Clone)]
pub struct TestHooks {
    pub sense_helper: SenseHelperHook,
    pub act_descriptor_helper: ActDescriptorHelperHook,
    pub primary: PrimaryHook,
    pub acts_helper: ActsHelperHook,
    pub goal_stack_helper: GoalStackHelperHook,
}

impl TestHooks {
    pub fn new(
        sense_helper: SenseHelperHook,
        act_descriptor_helper: ActDescriptorHelperHook,
        primary: PrimaryHook,
        acts_helper: ActsHelperHook,
        goal_stack_helper: GoalStackHelperHook,
    ) -> Self {
        Self {
            sense_helper,
            act_descriptor_helper,
            primary,
            acts_helper,
            goal_stack_helper,
        }
    }
}

pub fn cortex_with_hooks(hooks: TestHooks, limits: ReactionLimits) -> Cortex {
    Cortex::for_test_with_hooks(hooks, limits)
}
