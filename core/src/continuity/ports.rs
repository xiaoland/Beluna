use async_trait::async_trait;

use crate::{
    continuity::error::ContinuityError,
    spine::types::{AdmittedActionBatch, SpineCapabilityCatalog, SpineExecutionReport},
};

#[async_trait]
pub trait SpinePort: Send + Sync {
    async fn execute_admitted(
        &self,
        admitted: AdmittedActionBatch,
    ) -> Result<SpineExecutionReport, ContinuityError>;

    fn capability_catalog_snapshot(&self) -> SpineCapabilityCatalog;
}
