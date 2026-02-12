#![allow(dead_code)]

pub mod adapters;
pub mod clamp;
pub mod error;
pub mod pipeline;
pub mod ports;
pub mod reactor;
pub mod types;

pub use adapters::{AIGatewayAttemptExtractor, AIGatewayPayloadFiller, AIGatewayPrimaryReasoner};
pub use clamp::{DeterministicAttemptClamp, derive_attempt_id, derive_cost_attribution_id};
pub use error::{CortexError, CortexErrorKind};
pub use pipeline::CortexPipeline;
pub use ports::{
    AttemptClampPort, AttemptClampRequest, AttemptExtractorPort, AttemptExtractorRequest,
    CortexTelemetryEvent, CortexTelemetryPort, NoopTelemetryPort, PayloadFillerPort,
    PayloadFillerRequest, PrimaryReasonerPort, PrimaryReasonerRequest,
};
pub use reactor::CortexReactor;
pub use types::{
    AdmissionOutcomeSignal, AffordanceCapability, AttemptDraft, AttemptId, AttentionTag,
    CapabilityCatalog, ClampResult, ClampViolation, ClampViolationCode, ConstitutionalIntent,
    EmergentIntentCandidate, EndpointSnapshot, EnvironmentalIntentSignal, IntentContext, ProseIr,
    ReactionId, ReactionInput, ReactionLimits, ReactionResult, SenseDelta, SenseId,
};
