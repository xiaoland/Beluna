#![allow(dead_code)]

pub mod clamp;
pub mod contracts;
pub mod error;
pub mod runtime;
pub mod types;

pub use clamp::{AttemptClampRequest, DeterministicAttemptClamp, derive_act_id};
pub use contracts::{
    Act, ActId, CapabilityDropPatch, CapabilityPatch, RequestedResources, Sense, SenseDatum,
    SenseId, is_uuid_v4, is_uuid_v7,
};
pub use error::{CortexError, CortexErrorKind};
pub use runtime::{
    AttemptExtractorHook, AttemptExtractorRequest, Cortex, CortexTelemetryEvent,
    CortexTelemetryHook, PrimaryReasonerHook, PrimaryReasonerRequest,
};
pub use types::{
    AffordanceCapability, AttemptDraft, AttentionTag, CapabilityCatalog, ClampResult,
    ClampViolation, ClampViolationCode, CortexOutput, ProseIr, ReactionLimits,
};
