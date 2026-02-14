use async_trait::async_trait;

use crate::{
    cortex::{
        error::CortexError,
        types::{
            AttemptDraft, CapabilityCatalog, ClampResult, ClampViolation, CortexOutput, ProseIr,
            ReactionLimits,
        },
    },
    runtime_types::{CognitionState, PhysicalState, Sense, SenseId},
};

#[derive(Debug, Clone)]
pub struct PrimaryReasonerRequest {
    pub cycle_id: u64,
    pub senses: Vec<Sense>,
    pub physical_state: PhysicalState,
    pub cognition_state: CognitionState,
    pub limits: ReactionLimits,
}

#[derive(Debug, Clone)]
pub struct AttemptExtractorRequest {
    pub cycle_id: u64,
    pub prose_ir: ProseIr,
    pub capability_catalog: CapabilityCatalog,
    pub senses: Vec<Sense>,
    pub cognition_state: CognitionState,
    pub limits: ReactionLimits,
}

#[derive(Debug, Clone)]
pub struct PayloadFillerRequest {
    pub cycle_id: u64,
    pub drafts: Vec<AttemptDraft>,
    pub capability_catalog: CapabilityCatalog,
    pub clamp_violations: Vec<ClampViolation>,
    pub limits: ReactionLimits,
}

#[derive(Debug, Clone)]
pub struct AttemptClampRequest {
    pub cycle_id: u64,
    pub drafts: Vec<AttemptDraft>,
    pub capability_catalog: CapabilityCatalog,
    pub known_sense_ids: Vec<SenseId>,
    pub limits: ReactionLimits,
}

#[async_trait]
pub trait PrimaryReasonerPort: Send + Sync {
    async fn infer_ir(&self, req: PrimaryReasonerRequest) -> Result<ProseIr, CortexError>;
}

#[async_trait]
pub trait AttemptExtractorPort: Send + Sync {
    async fn extract(&self, req: AttemptExtractorRequest)
    -> Result<Vec<AttemptDraft>, CortexError>;
}

#[async_trait]
pub trait PayloadFillerPort: Send + Sync {
    async fn fill(&self, req: PayloadFillerRequest) -> Result<Vec<AttemptDraft>, CortexError>;
}

pub trait AttemptClampPort: Send + Sync {
    fn clamp(&self, req: AttemptClampRequest) -> Result<ClampResult, CortexError>;
}

#[async_trait]
pub trait CortexPort: Send + Sync {
    async fn cortex(
        &self,
        senses: &[Sense],
        physical_state: &PhysicalState,
        cognition_state: &CognitionState,
    ) -> Result<CortexOutput, CortexError>;
}

#[derive(Debug, Clone)]
pub enum CortexTelemetryEvent {
    ReactionStarted { cycle_id: u64 },
    StageFailed { cycle_id: u64, stage: &'static str },
    ReactionCompleted { cycle_id: u64, act_count: usize },
    NoopFallback { cycle_id: u64, reason: &'static str },
}

pub trait CortexTelemetryPort: Send + Sync {
    fn on_event(&self, event: CortexTelemetryEvent);
}

#[derive(Default)]
pub struct NoopTelemetryPort;

impl CortexTelemetryPort for NoopTelemetryPort {
    fn on_event(&self, _event: CortexTelemetryEvent) {}
}
