use std::{future::Future, pin::Pin, sync::Arc};

use crate::{
    cortex::{Cortex, CortexError, ReactionLimits},
    types::{NeuralSignalDescriptor, Sense},
};

#[derive(Debug, Clone)]
pub struct SenseHelperRequest {
    pub cycle_id: u64,
    pub senses: Vec<Sense>,
    pub sense_descriptors: Vec<NeuralSignalDescriptor>,
}

#[derive(Debug, Clone)]
pub struct GoalForestHelperRequest {
    pub cycle_id: u64,
    pub goal_forest_json: String,
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestActDraft {
    pub endpoint_id: String,
    pub fq_act_id: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

pub type TestActsHelperOutput = Vec<TestActDraft>;

type SenseHelperFuture = Pin<Box<dyn Future<Output = Result<String, CortexError>> + Send>>;
type GoalForestHelperFuture = Pin<Box<dyn Future<Output = Result<String, CortexError>> + Send>>;
type PrimaryFuture = Pin<Box<dyn Future<Output = Result<String, CortexError>> + Send>>;
type ActsHelperFuture =
    Pin<Box<dyn Future<Output = Result<TestActsHelperOutput, CortexError>> + Send>>;

pub type SenseHelperHook = Arc<dyn Fn(SenseHelperRequest) -> SenseHelperFuture + Send + Sync>;
pub type GoalForestHelperHook =
    Arc<dyn Fn(GoalForestHelperRequest) -> GoalForestHelperFuture + Send + Sync>;
pub type PrimaryHook = Arc<dyn Fn(PrimaryRequest) -> PrimaryFuture + Send + Sync>;
pub type ActsHelperHook = Arc<dyn Fn(ActsHelperRequest) -> ActsHelperFuture + Send + Sync>;

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
    pub goal_forest_helper: GoalForestHelperHook,
    pub primary: PrimaryHook,
    pub acts_helper: ActsHelperHook,
}

impl TestHooks {
    pub fn new(
        sense_helper: SenseHelperHook,
        goal_forest_helper: GoalForestHelperHook,
        primary: PrimaryHook,
        acts_helper: ActsHelperHook,
    ) -> Self {
        Self {
            sense_helper,
            goal_forest_helper,
            primary,
            acts_helper,
        }
    }
}

pub fn cortex_with_hooks(hooks: TestHooks, limits: ReactionLimits) -> Cortex {
    Cortex::for_test_with_hooks(hooks, limits)
}
