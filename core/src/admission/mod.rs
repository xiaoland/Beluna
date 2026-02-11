#![allow(dead_code)]

pub mod affordance;
pub mod resolver;
pub mod types;

pub use affordance::{AffordanceProfile, AffordanceRegistry, DegradationProfile};
pub use resolver::{
    AdmissionContext, AdmissionResolver, AdmissionResolverConfig, CostAdmissionPolicy,
    DegradationPreference, derive_action_id,
};
pub use types::{
    AdmissionDisposition, AdmissionReport, AdmissionReportItem, AdmissionWhy,
    AffordabilitySnapshot, AttributionRecord, CommitmentId, ConstraintCode, CostAttributionId,
    CycleId, EconomicCode, GoalId, IntentAttempt, MetadataValue, RequestedResources,
    ReservationDelta,
};
