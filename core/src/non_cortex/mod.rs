#![allow(dead_code)]

pub mod affordance;
pub mod continuity;
pub mod debit_sources;
pub mod error;
pub mod facade;
pub mod ledger;
pub mod noop;
pub mod ports;
pub mod resolver;
pub mod types;

pub use affordance::{AffordanceProfile, AffordanceRegistry, DegradationProfile};
pub use continuity::assert_settlement_consistency;
pub use debit_sources::{AIGatewayApproxDebitSource, ExternalDebitSourcePort, InMemoryDebitSource};
pub use error::{NonCortexError, NonCortexErrorKind};
pub use facade::NonCortexFacade;
pub use ledger::{
    LedgerEntry, LedgerEntryKind, ReservationRecord, ReservationState, SurvivalLedger,
};
pub use noop::{NoopDebitSource, SpinePortAdapter};
pub use ports::SpinePort;
pub use resolver::{
    AdmissionResolver, AdmissionResolverConfig, CostAdmissionPolicy, DegradationPreference,
};
pub use types::{
    AdmissionDisposition, AdmissionReport, AdmissionReportItem, AdmissionWhy,
    AffordabilitySnapshot, AttributionRecord, CommitmentId, ConstraintCode, CostAttributionId,
    CycleId, EconomicCode, ExternalDebitObservation, GoalId, IntentAttempt, LedgerEntryId,
    MetadataValue, NonCortexCycleOutput, NonCortexState, RequestedResources, ReservationDelta,
};
