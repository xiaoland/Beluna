use std::sync::Arc;

use crate::{
    continuity::{
        debit_sources::ExternalDebitSourcePort,
        error::{ContinuityError, ContinuityErrorKind},
        ports::SpinePort,
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
    ) -> Result<crate::spine::types::SpineExecutionReport, ContinuityError> {
        self.inner
            .execute_admitted(admitted)
            .map_err(|err| ContinuityError::new(ContinuityErrorKind::Internal, err.to_string()))
    }
}

#[derive(Default)]
pub struct NoopDebitSource;

impl ExternalDebitSourcePort for NoopDebitSource {
    fn drain_observations(&self) -> Vec<ExternalDebitObservation> {
        Vec::new()
    }
}
