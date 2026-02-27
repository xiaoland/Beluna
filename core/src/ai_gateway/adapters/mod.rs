use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;

use crate::ai_gateway::{
    chat::types::{AdapterInvocation, BackendCompleteResponse, TurnPayload},
    error::GatewayError,
    types::{AdapterContext, BackendCapabilities, BackendDialect},
};

pub mod github_copilot;
pub(crate) mod http_errors;
pub(crate) mod http_stream;
pub mod ollama;
pub mod openai_compatible;
pub(crate) mod wire;

#[async_trait]
pub(crate) trait BackendAdapter: Send + Sync {
    fn dialect(&self) -> BackendDialect;
    fn static_capabilities(&self) -> BackendCapabilities;
    fn supports_tool_retry(&self) -> bool {
        false
    }

    async fn complete(
        &self,
        _ctx: AdapterContext,
        _payload: &TurnPayload,
    ) -> Result<BackendCompleteResponse, GatewayError> {
        Err(GatewayError::new(
            crate::ai_gateway::error::GatewayErrorKind::UnsupportedCapability,
            "adapter does not implement chat complete",
        )
        .with_retryable(false))
    }

    async fn stream(
        &self,
        ctx: AdapterContext,
        payload: &TurnPayload,
    ) -> Result<AdapterInvocation, GatewayError>;
}

pub(crate) fn build_default_adapters() -> HashMap<BackendDialect, Arc<dyn BackendAdapter>> {
    let mut adapters: HashMap<BackendDialect, Arc<dyn BackendAdapter>> = HashMap::new();
    adapters.insert(
        BackendDialect::OpenAiCompatible,
        Arc::new(openai_compatible::OpenAiCompatibleAdapter::default()),
    );
    adapters.insert(
        BackendDialect::Ollama,
        Arc::new(ollama::OllamaAdapter::default()),
    );
    adapters.insert(
        BackendDialect::GitHubCopilotSdk,
        Arc::new(github_copilot::GitHubCopilotAdapter::default()),
    );
    adapters
}
