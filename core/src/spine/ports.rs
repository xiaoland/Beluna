use crate::spine::{
    error::SpineError,
    types::{AdmittedActionBatch, SpineExecutionMode, SpineExecutionReport},
};

pub trait SpineExecutorPort: Send + Sync {
    fn mode(&self) -> SpineExecutionMode;

    fn execute_admitted(
        &self,
        admitted: AdmittedActionBatch,
    ) -> Result<SpineExecutionReport, SpineError>;
}
