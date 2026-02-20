#![allow(dead_code)]

use std::sync::{Arc, OnceLock};

pub mod adapters;
pub mod endpoint;
pub mod error;
pub mod runtime;
pub mod types;

pub use endpoint::{Endpoint, NativeFunctionEndpoint};
pub use error::{SpineError, SpineErrorKind};
pub use runtime::{EndpointBinding, Spine, shutdown_global_spine};
pub use types::{
    ActDispatchResult, CostAttributionId, EndpointExecutionOutcome, NeuralSignalDescriptor,
    NeuralSignalDescriptorCatalog, NeuralSignalDescriptorRouteKey, ReserveEntryId, SpineEvent,
    SpineExecutionMode,
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
