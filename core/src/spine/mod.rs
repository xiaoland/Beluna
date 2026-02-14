#![allow(dead_code)]

use std::sync::{Arc, OnceLock};

pub mod adapters;
pub mod error;
pub mod noop;
pub mod ports;
pub mod registry;
pub mod router;
pub mod types;

pub use error::{SpineError, SpineErrorKind};
pub use noop::DeterministicNoopSpine;
pub use ports::{EndpointPort, EndpointRegistryPort, SpineExecutorPort};
pub use registry::InMemoryEndpointRegistry;
pub use router::{NativeFunctionEndpoint, RoutingSpineExecutor};
pub use types::{
    CostAttributionId, CostVector, EndpointCapabilityDescriptor, EndpointExecutionOutcome,
    ReserveEntryId, RouteKey, SpineCapabilityCatalog, SpineEvent, SpineExecutionMode,
};

static GLOBAL_SPINE_EXECUTOR: OnceLock<Arc<dyn SpineExecutorPort>> = OnceLock::new();

pub fn install_global_executor(executor: Arc<dyn SpineExecutorPort>) -> Result<(), SpineError> {
    GLOBAL_SPINE_EXECUTOR
        .set(executor)
        .map_err(|_| error::internal_error("spine executor is already initialized"))
}

pub fn global_executor() -> Option<Arc<dyn SpineExecutorPort>> {
    GLOBAL_SPINE_EXECUTOR.get().cloned()
}
