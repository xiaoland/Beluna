#![allow(dead_code)]

pub mod adapters;
pub mod clamp;
pub mod contracts;
pub mod error;
pub mod pipeline;
pub mod ports;
pub mod types;

pub use adapters::{AIGatewayAttemptExtractor, AIGatewayPayloadFiller, AIGatewayPrimaryReasoner};
pub use clamp::{DeterministicAttemptClamp, derive_act_id};
pub use contracts::{
    Act, ActId, CapabilityDropPatch, CapabilityPatch, RequestedResources, Sense, SenseDatum,
    SenseId, is_uuid_v4, is_uuid_v7,
};
pub use error::{CortexError, CortexErrorKind};
pub use pipeline::CortexPipeline;
pub use ports::{
    AttemptClampPort, AttemptClampRequest, AttemptExtractorPort, AttemptExtractorRequest,
    CortexPort, CortexTelemetryEvent, CortexTelemetryPort, NoopTelemetryPort, PayloadFillerPort,
    PayloadFillerRequest, PrimaryReasonerPort, PrimaryReasonerRequest,
};
pub use types::{
    AffordanceCapability, AttemptDraft, AttentionTag, CapabilityCatalog, ClampResult,
    ClampViolation, ClampViolationCode, CortexOutput, ProseIr, ReactionLimits,
};
