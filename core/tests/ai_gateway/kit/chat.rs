use std::sync::Arc;

use async_trait::async_trait;
use beluna::ai_gateway::{
    chat::{
        Chat, ChatMessage, ChatRole, ContentPart, ToolExecutionRequest, ToolExecutionResult,
        ToolExecutor,
    },
    credentials::EnvCredentialProvider,
    types::{
        AIGatewayConfig, BackendDialect, BackendProfile, ChatConfig, CredentialRef, ModelProfile,
        ResilienceConfig,
    },
};
use serde_json::{Value, json};

pub fn chat_for_responses_endpoint(endpoint: String) -> Chat {
    Chat::new(
        &AIGatewayConfig {
            backends: vec![BackendProfile {
                id: "openai".to_string(),
                dialect: BackendDialect::OpenAiResponses,
                endpoint: Some(endpoint),
                credential: CredentialRef::None,
                models: vec![ModelProfile {
                    id: "gpt-5".to_string(),
                    aliases: vec!["default".to_string()],
                }],
                capabilities: None,
                copilot: None,
            }],
            chat: ChatConfig::default(),
            resilience: ResilienceConfig::default(),
        },
        Arc::new(EnvCredentialProvider),
    )
    .expect("chat")
}

pub fn user_message(text: &str) -> ChatMessage {
    ChatMessage {
        role: ChatRole::User,
        parts: vec![ContentPart::Text {
            text: text.to_string(),
        }],
        tool_call_id: None,
        tool_name: None,
        tool_calls: Vec::new(),
    }
}

pub fn text_response(text: &str) -> Value {
    json!({
        "status": "completed",
        "output": [{
            "type": "message",
            "role": "assistant",
            "content": [{
                "type": "output_text",
                "text": text
            }]
        }]
    })
}

#[derive(Clone)]
pub struct EchoToolExecutor;

#[async_trait]
impl ToolExecutor for EchoToolExecutor {
    async fn execute_call(
        &self,
        _request: ToolExecutionRequest,
    ) -> Result<ToolExecutionResult, beluna::ai_gateway::error::GatewayError> {
        Ok(ToolExecutionResult {
            payload: json!({"ok": true}),
            reset_messages_applied: false,
        })
    }
}
