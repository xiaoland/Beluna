#![allow(dead_code)]

pub mod debit_sources;
pub mod engine;
pub mod error;
pub mod invariants;
pub mod noop;
pub mod ports;
pub mod state;
pub mod types;

pub use debit_sources::{AIGatewayApproxDebitSource, ExternalDebitSourcePort, InMemoryDebitSource};
pub use engine::ContinuityEngine;
pub use error::{ContinuityError, ContinuityErrorKind};
pub use invariants::assert_settlement_consistency;
pub use noop::{NoopDebitSource, SpinePortAdapter};
pub use ports::SpinePort;
pub use state::ContinuityState;
pub use types::{ContinuityCycleOutput, ExternalDebitObservation, SenseSample, SituationView};
