use std::sync::Arc;

use async_trait::async_trait;

use crate::spine::{
    error::SpineError,
    types::{
        EndpointCapabilityDescriptor, EndpointExecutionOutcome, RouteKey, SpineCapabilityCatalog,
        SpineExecutionMode,
    },
};

#[async_trait]
pub trait EndpointPort: Send + Sync {
    async fn invoke(
        &self,
        act: crate::runtime_types::Act,
    ) -> Result<EndpointExecutionOutcome, SpineError>;
}

pub trait EndpointRegistryPort: Send + Sync {
    fn register(
        &self,
        registration: EndpointCapabilityDescriptor,
        endpoint: Arc<dyn EndpointPort>,
    ) -> Result<(), SpineError>;

    fn unregister(&self, route: &RouteKey) -> Option<EndpointCapabilityDescriptor>;

    fn resolve(&self, endpoint_id: &str) -> Option<Arc<dyn EndpointPort>>;

    fn catalog_snapshot(&self) -> SpineCapabilityCatalog;
}

#[async_trait]
pub trait SpineExecutorPort: Send + Sync {
    fn mode(&self) -> SpineExecutionMode;

    async fn dispatch_act(
        &self,
        act: crate::runtime_types::Act,
    ) -> Result<EndpointExecutionOutcome, SpineError>;

    fn capability_catalog_snapshot(&self) -> SpineCapabilityCatalog;
}
