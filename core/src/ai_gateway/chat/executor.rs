use async_trait::async_trait;

use crate::ai_gateway::error::GatewayError;

use super::types::ToolCallResult;

#[derive(Debug, Clone)]
pub struct ToolExecutionRequest {
    pub chat_id: String,
    pub thread_id: String,
    pub turn_id: u64,
    pub call: ToolCallResult,
}

#[derive(Debug, Clone)]
pub struct ToolExecutionResult {
    pub payload: serde_json::Value,
    pub reset_messages_applied: bool,
}

#[async_trait]
pub trait ToolExecutor: Send + Sync {
    async fn execute_call(
        &self,
        request: ToolExecutionRequest,
    ) -> Result<ToolExecutionResult, GatewayError>;
}
