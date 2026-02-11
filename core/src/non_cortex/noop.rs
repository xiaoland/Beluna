use std::sync::Arc;

use crate::{
    non_cortex::{
        debit_sources::ExternalDebitSourcePort, error::NonCortexError, ports::SpinePort,
        types::ExternalDebitObservation,
    },
    spine::{ports::SpineExecutorPort, types::AdmittedActionBatch},
};

pub struct SpinePortAdapter {
    inner: Arc<dyn SpineExecutorPort>,
}

impl SpinePortAdapter {
    pub fn new(inner: Arc<dyn SpineExecutorPort>) -> Self {
        Self { inner }
    }
}

impl SpinePort for SpinePortAdapter {
    fn execute_admitted(
        &self,
        admitted: AdmittedActionBatch,
    ) -> Result<crate::spine::types::SpineExecutionReport, NonCortexError> {
        self.inner.execute_admitted(admitted).map_err(|err| {
            NonCortexError::new(
                crate::non_cortex::error::NonCortexErrorKind::Internal,
                err.to_string(),
            )
        })
    }
}

#[derive(Default)]
pub struct NoopDebitSource;

impl ExternalDebitSourcePort for NoopDebitSource {
    fn drain_observations(&self) -> Vec<ExternalDebitObservation> {
        Vec::new()
    }
}
