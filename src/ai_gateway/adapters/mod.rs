use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;

use crate::ai_gateway::{
    error::GatewayError,
    types::{
        AdapterContext, AdapterInvocation, BackendCapabilities, BackendDialect, CanonicalRequest,
    },
};

pub mod copilot_rpc;
pub mod github_copilot;
pub mod http_common;
pub mod ollama;
pub mod openai_compatible;

#[async_trait]
pub trait BackendAdapter: Send + Sync {
    fn dialect(&self) -> BackendDialect;
    fn static_capabilities(&self) -> BackendCapabilities;
    fn supports_tool_retry(&self) -> bool {
        false
    }

    async fn invoke_stream(
        &self,
        ctx: AdapterContext,
        req: CanonicalRequest,
    ) -> Result<AdapterInvocation, GatewayError>;
}

pub fn build_default_adapters() -> HashMap<BackendDialect, Arc<dyn BackendAdapter>> {
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
