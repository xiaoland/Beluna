#![allow(dead_code)]

use std::sync::{Arc, OnceLock};

pub mod adapters;
pub mod error;
pub mod noop;
pub mod ports;
pub mod registry;
pub mod router;
pub mod runtime;
pub mod types;

pub use error::{SpineError, SpineErrorKind};
pub use noop::DeterministicNoopSpine;
pub use ports::{EndpointPort, EndpointRegistryPort, SpineExecutorPort};
pub use registry::InMemoryEndpointRegistry;
pub use router::{NativeFunctionEndpoint, RoutingSpineExecutor};
pub use runtime::{Spine, SpineHandle, shutdown_global_spine};
pub use types::{
    CostAttributionId, CostVector, EndpointCapabilityDescriptor, EndpointExecutionOutcome,
    ReserveEntryId, RouteKey, SpineCapabilityCatalog, SpineEvent, SpineExecutionMode,
};

static GLOBAL_SPINE: OnceLock<Arc<Spine>> = OnceLock::new();

pub fn install_global_spine(spine: Arc<Spine>) -> Result<(), SpineError> {
    GLOBAL_SPINE
        .set(spine)
        .map_err(|_| error::internal_error("spine singleton is already initialized"))
}

pub fn global_spine() -> Option<Arc<Spine>> {
    GLOBAL_SPINE.get().cloned()
}

pub fn global_executor() -> Option<Arc<dyn SpineExecutorPort>> {
    global_spine().map(|spine| spine.executor_port())
}
