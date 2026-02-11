use crate::{
    non_cortex::error::NonCortexError,
    spine::types::{AdmittedActionBatch, SpineExecutionReport},
};

pub trait SpinePort: Send + Sync {
    fn execute_admitted(
        &self,
        admitted: AdmittedActionBatch,
    ) -> Result<SpineExecutionReport, NonCortexError>;
}
