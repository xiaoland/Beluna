use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    continuity::{
        debit_sources::ExternalDebitSourcePort,
        error::{ContinuityError, ContinuityErrorKind},
        ports::SpinePort,
        types::ExternalDebitObservation,
    },
    spine::{
        ports::SpineExecutorPort,
        types::{AdmittedActionBatch, SpineCapabilityCatalog},
    },
};

pub struct SpinePortAdapter {
    inner: Arc<dyn SpineExecutorPort>,
}

impl SpinePortAdapter {
    pub fn new(inner: Arc<dyn SpineExecutorPort>) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl SpinePort for SpinePortAdapter {
    async fn execute_admitted(
        &self,
        admitted: AdmittedActionBatch,
    ) -> Result<crate::spine::types::SpineExecutionReport, ContinuityError> {
        self.inner
            .execute_admitted(admitted)
            .await
            .map_err(|err| ContinuityError::new(ContinuityErrorKind::Internal, err.to_string()))
    }

    fn capability_catalog_snapshot(&self) -> SpineCapabilityCatalog {
        self.inner.capability_catalog_snapshot()
    }
}

#[derive(Default)]
pub struct NoopDebitSource;

impl ExternalDebitSourcePort for NoopDebitSource {
    fn drain_observations(&self) -> Vec<ExternalDebitObservation> {
        Vec::new()
    }
}
