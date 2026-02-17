use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    spine::{error::SpineError, types::EndpointExecutionOutcome},
    types::Act,
};

type NativeEndpointHandler =
    dyn Fn(Act) -> Result<EndpointExecutionOutcome, SpineError> + Send + Sync;

#[async_trait]
pub trait Endpoint: Send + Sync {
    async fn invoke(&self, act: Act) -> Result<EndpointExecutionOutcome, SpineError>;
}

pub struct NativeFunctionEndpoint {
    handler: Arc<NativeEndpointHandler>,
}

impl NativeFunctionEndpoint {
    pub fn new(handler: Arc<NativeEndpointHandler>) -> Self {
        Self { handler }
    }
}

#[async_trait]
impl Endpoint for NativeFunctionEndpoint {
    async fn invoke(&self, act: Act) -> Result<EndpointExecutionOutcome, SpineError> {
        (self.handler)(act)
    }
}
