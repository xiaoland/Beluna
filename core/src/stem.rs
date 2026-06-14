pub mod afferent_pathway;
pub mod efferent_pathway;
pub mod pathway;
pub mod runtime;

pub use afferent_pathway::{
    AfferentControlHandle, AfferentDispatchResult, AfferentMiddleware, AfferentMiddlewareDecision,
    AfferentPathwayError, AfferentPathwayErrorKind, SenseAfferentPathway, SenseConsumerHandle,
    SenseIngressHandle,
};
pub use efferent_pathway::{
    ActProducerHandle, ContinuityEfferentMiddleware, EfferentActEnvelope, EfferentEnqueueError,
    EfferentMiddleware, EfferentMiddlewareContext, EfferentMiddlewareDecision, EfferentTx,
    SpineEfferentMiddleware, new_efferent_pathway, spawn_efferent_runtime,
};
pub use pathway::{ContinueOutput, PathwayMiddlewareDecision};
pub use runtime::{StemControlPort, StemDeps, StemPhysicalStateStore, StemTickRuntime, TickGrant};
