#![allow(dead_code)]

pub mod adapters;
pub mod clamp;
pub mod error;
pub mod pipeline;
pub mod ports;
pub mod types;

pub use adapters::{AIGatewayAttemptExtractor, AIGatewayPayloadFiller, AIGatewayPrimaryReasoner};
pub use clamp::{DeterministicAttemptClamp, derive_act_id};
pub use error::{CortexError, CortexErrorKind};
pub use pipeline::CortexPipeline;
pub use ports::{
    AttemptClampPort, AttemptClampRequest, AttemptExtractorPort, AttemptExtractorRequest, CortexPort,
    CortexTelemetryEvent, CortexTelemetryPort, NoopTelemetryPort, PayloadFillerPort, PayloadFillerRequest,
    PrimaryReasonerPort, PrimaryReasonerRequest,
};
pub use types::{
    AffordanceCapability, AttemptDraft, AttentionTag, CapabilityCatalog, ClampResult,
    ClampViolation, ClampViolationCode, CortexOutput, ProseIr, ReactionLimits,
};
