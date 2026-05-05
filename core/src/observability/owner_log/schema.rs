use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OwnerScope {
    MainRuntime,
    StemTick,
    CortexPrimary,
    CortexAttention,
    CortexCleanup,
    CortexSenseHelper,
    CortexGoalForest,
    CortexActsHelper,
    AiGatewayTransport,
    AiGatewayChatTurn,
    AiGatewayChatThread,
    SpineActRouting,
}

impl OwnerScope {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            OwnerScope::MainRuntime => "beluna.core.main.runtime",
            OwnerScope::StemTick => "beluna.core.stem.tick",
            OwnerScope::CortexPrimary => "beluna.core.cortex.primary",
            OwnerScope::CortexAttention => "beluna.core.cortex.attention",
            OwnerScope::CortexCleanup => "beluna.core.cortex.cleanup",
            OwnerScope::CortexSenseHelper => "beluna.core.cortex.sense-helper",
            OwnerScope::CortexGoalForest => "beluna.core.cortex.goal-forest",
            OwnerScope::CortexActsHelper => "beluna.core.cortex.acts-helper",
            OwnerScope::AiGatewayTransport => "beluna.core.ai-gateway.transport",
            OwnerScope::AiGatewayChatTurn => "beluna.core.ai-gateway.chat.turn",
            OwnerScope::AiGatewayChatThread => "beluna.core.ai-gateway.chat.thread",
            OwnerScope::SpineActRouting => "beluna.core.spine.act-routing",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrganResponseStatus {
    Ok,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DispatchOutcomeClass {
    Acknowledged,
    Rejected,
    Lost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DescriptorCatalogChangeMode {
    Snapshot,
    Update,
    Drop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterLifecycleState {
    Enabled,
    Disabled,
    Faulted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndpointLifecycleTransition {
    Connected,
    Disconnected,
    Registered,
    Dropped,
}
