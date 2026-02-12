use std::sync::Arc;

use async_trait::async_trait;

use crate::spine::{
    error::SpineError,
    types::{
        AdmittedActionBatch, EndpointExecutionOutcome, EndpointInvocation, EndpointRegistration,
        RouteKey, SpineCapabilityCatalog, SpineExecutionMode, SpineExecutionReport,
    },
};

#[async_trait]
pub trait EndpointPort: Send + Sync {
    async fn invoke(
        &self,
        invocation: EndpointInvocation,
    ) -> Result<EndpointExecutionOutcome, SpineError>;
}

pub trait EndpointRegistryPort: Send + Sync {
    fn register(
        &self,
        registration: EndpointRegistration,
        endpoint: Arc<dyn EndpointPort>,
    ) -> Result<(), SpineError>;

    fn unregister(&self, route: &RouteKey) -> Option<EndpointRegistration>;

    fn resolve(&self, route: &RouteKey) -> Option<Arc<dyn EndpointPort>>;

    fn catalog_snapshot(&self) -> SpineCapabilityCatalog;
}

#[async_trait]
pub trait SpineExecutorPort: Send + Sync {
    fn mode(&self) -> SpineExecutionMode;

    async fn execute_admitted(
        &self,
        admitted: AdmittedActionBatch,
    ) -> Result<SpineExecutionReport, SpineError>;

    fn capability_catalog_snapshot(&self) -> SpineCapabilityCatalog;
}
