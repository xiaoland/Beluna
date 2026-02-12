use async_trait::async_trait;

use crate::cortex::{
    error::CortexError,
    types::{
        AttemptDraft, CapabilityCatalog, ClampResult, ClampViolation, ProseIr, ReactionId,
        ReactionLimits, SenseDelta,
    },
};

#[derive(Debug, Clone)]
pub struct PrimaryReasonerRequest {
    pub reaction_id: ReactionId,
    pub prompt_context: String,
    pub sense_window: Vec<SenseDelta>,
    pub limits: ReactionLimits,
}

#[derive(Debug, Clone)]
pub struct AttemptExtractorRequest {
    pub reaction_id: ReactionId,
    pub prose_ir: ProseIr,
    pub capability_catalog: CapabilityCatalog,
    pub sense_window: Vec<SenseDelta>,
    pub limits: ReactionLimits,
}

#[derive(Debug, Clone)]
pub struct PayloadFillerRequest {
    pub reaction_id: ReactionId,
    pub drafts: Vec<AttemptDraft>,
    pub capability_catalog: CapabilityCatalog,
    pub clamp_violations: Vec<ClampViolation>,
    pub limits: ReactionLimits,
}

#[derive(Debug, Clone)]
pub struct AttemptClampRequest {
    pub reaction_id: ReactionId,
    pub drafts: Vec<AttemptDraft>,
    pub capability_catalog: CapabilityCatalog,
    pub sense_window: Vec<SenseDelta>,
    pub limits: ReactionLimits,
}

#[async_trait]
pub trait PrimaryReasonerPort: Send + Sync {
    async fn infer_ir(&self, req: PrimaryReasonerRequest) -> Result<ProseIr, CortexError>;
}

#[async_trait]
pub trait AttemptExtractorPort: Send + Sync {
    async fn extract(&self, req: AttemptExtractorRequest) -> Result<Vec<AttemptDraft>, CortexError>;
}

#[async_trait]
pub trait PayloadFillerPort: Send + Sync {
    async fn fill(&self, req: PayloadFillerRequest) -> Result<Vec<AttemptDraft>, CortexError>;
}

pub trait AttemptClampPort: Send + Sync {
    fn clamp(&self, req: AttemptClampRequest) -> Result<ClampResult, CortexError>;
}

#[derive(Debug, Clone)]
pub enum CortexTelemetryEvent {
    ReactionStarted {
        reaction_id: ReactionId,
    },
    StageFailed {
        reaction_id: ReactionId,
        stage: &'static str,
    },
    ReactionCompleted {
        reaction_id: ReactionId,
        attempt_count: usize,
    },
    NoopFallback {
        reaction_id: ReactionId,
        reason: &'static str,
    },
}

pub trait CortexTelemetryPort: Send + Sync {
    fn on_event(&self, event: CortexTelemetryEvent);
}

#[derive(Default)]
pub struct NoopTelemetryPort;

impl CortexTelemetryPort for NoopTelemetryPort {
    fn on_event(&self, _event: CortexTelemetryEvent) {}
}
