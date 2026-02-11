use crate::{
    continuity::error::ContinuityError,
    spine::types::{AdmittedActionBatch, SpineExecutionReport},
};

pub trait SpinePort: Send + Sync {
    fn execute_admitted(
        &self,
        admitted: AdmittedActionBatch,
    ) -> Result<SpineExecutionReport, ContinuityError>;
}
