use std::sync::Arc;

use async_trait::async_trait;

use crate::spine::{
    error::SpineError,
    types::{
        ActDispatchRequest, EndpointExecutionOutcome, EndpointInvocation, EndpointRegistration,
        RouteKey, SpineCapabilityCatalog, SpineEvent, SpineExecutionMode,
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

    async fn dispatch_act(&self, request: ActDispatchRequest) -> Result<SpineEvent, SpineError>;

    fn capability_catalog_snapshot(&self) -> SpineCapabilityCatalog;
}
