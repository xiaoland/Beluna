#![allow(dead_code)]

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
    ActDispatchRequest, CostAttributionId, CostVector, EndpointCapabilityDescriptor,
    EndpointExecutionOutcome, EndpointInvocation, EndpointRegistration, ReserveEntryId, RouteKey,
    SpineCapabilityCatalog, SpineEvent, SpineExecutionMode,
};
