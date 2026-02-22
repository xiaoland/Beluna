use std::{future::Future, pin::Pin, sync::Arc};

use crate::{
    cortex::{CognitionState, Cortex, CortexError, GoalTreePatchOp, ReactionLimits},
    types::{NeuralSignalDescriptor, Sense},
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
pub struct GoalTreeHelperRequest {
    pub cycle_id: u64,
    pub user_partition_json: String,
}

#[derive(Debug, Clone)]
pub struct PrimaryRequest {
    pub cycle_id: u64,
    pub input_ir: String,
}

#[derive(Debug, Clone)]
pub struct ActsHelperRequest {
    pub cycle_id: u64,
    pub acts_section: String,
}

#[derive(Debug, Clone)]
pub struct GoalTreePatchHelperRequest {
    pub cycle_id: u64,
    pub goal_tree_patch_section: String,
    pub cognition_state: CognitionState,
}

#[derive(Debug, Clone)]
pub struct L1MemoryFlushHelperRequest {
    pub cycle_id: u64,
    pub l1_memory_flush_section: String,
    pub cognition_state: CognitionState,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestActDraft {
    pub endpoint_id: String,
    pub neural_signal_descriptor_id: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

pub type TestActsHelperOutput = Vec<TestActDraft>;
pub type TestGoalTreePatchOutput = Vec<GoalTreePatchOp>;
pub type TestL1MemoryFlushOutput = Vec<String>;

type SenseHelperFuture = Pin<Box<dyn Future<Output = Result<String, CortexError>> + Send>>;
type ActDescriptorHelperFuture = Pin<Box<dyn Future<Output = Result<String, CortexError>> + Send>>;
type GoalTreeHelperFuture = Pin<Box<dyn Future<Output = Result<String, CortexError>> + Send>>;
type PrimaryFuture = Pin<Box<dyn Future<Output = Result<String, CortexError>> + Send>>;
type ActsHelperFuture =
    Pin<Box<dyn Future<Output = Result<TestActsHelperOutput, CortexError>> + Send>>;
type GoalTreePatchHelperFuture =
    Pin<Box<dyn Future<Output = Result<TestGoalTreePatchOutput, CortexError>> + Send>>;
type L1MemoryFlushHelperFuture =
    Pin<Box<dyn Future<Output = Result<TestL1MemoryFlushOutput, CortexError>> + Send>>;

pub type SenseHelperHook = Arc<dyn Fn(SenseHelperRequest) -> SenseHelperFuture + Send + Sync>;
pub type ActDescriptorHelperHook =
    Arc<dyn Fn(ActDescriptorHelperRequest) -> ActDescriptorHelperFuture + Send + Sync>;
pub type GoalTreeHelperHook =
    Arc<dyn Fn(GoalTreeHelperRequest) -> GoalTreeHelperFuture + Send + Sync>;
pub type PrimaryHook = Arc<dyn Fn(PrimaryRequest) -> PrimaryFuture + Send + Sync>;
pub type ActsHelperHook = Arc<dyn Fn(ActsHelperRequest) -> ActsHelperFuture + Send + Sync>;
pub type GoalTreePatchHelperHook =
    Arc<dyn Fn(GoalTreePatchHelperRequest) -> GoalTreePatchHelperFuture + Send + Sync>;
pub type L1MemoryFlushHelperHook =
    Arc<dyn Fn(L1MemoryFlushHelperRequest) -> L1MemoryFlushHelperFuture + Send + Sync>;

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
    pub goal_tree_helper: GoalTreeHelperHook,
    pub primary: PrimaryHook,
    pub acts_helper: ActsHelperHook,
    pub goal_tree_patch_helper: GoalTreePatchHelperHook,
    pub l1_memory_flush_helper: L1MemoryFlushHelperHook,
}

impl TestHooks {
    pub fn new(
        sense_helper: SenseHelperHook,
        act_descriptor_helper: ActDescriptorHelperHook,
        goal_tree_helper: GoalTreeHelperHook,
        primary: PrimaryHook,
        acts_helper: ActsHelperHook,
        goal_tree_patch_helper: GoalTreePatchHelperHook,
        l1_memory_flush_helper: L1MemoryFlushHelperHook,
    ) -> Self {
        Self {
            sense_helper,
            act_descriptor_helper,
            goal_tree_helper,
            primary,
            acts_helper,
            goal_tree_patch_helper,
            l1_memory_flush_helper,
        }
    }
}

pub fn cortex_with_hooks(hooks: TestHooks, limits: ReactionLimits) -> Cortex {
    Cortex::for_test_with_hooks(hooks, limits)
}
