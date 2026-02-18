use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    spine::{error::SpineError, types::ActDispatchResult},
    types::Act,
};

type NativeEndpointHandler = dyn Fn(Act) -> Result<ActDispatchResult, SpineError> + Send + Sync;

#[async_trait]
pub trait Endpoint: Send + Sync {
    async fn invoke(&self, act: Act) -> Result<ActDispatchResult, SpineError>;
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
    async fn invoke(&self, act: Act) -> Result<ActDispatchResult, SpineError> {
        (self.handler)(act)
    }
}
