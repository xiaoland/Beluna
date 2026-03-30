mod validate;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use validate::ContractValidationError;

pub const FIXTURE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObservabilitySubsystem {
    AiGateway,
    Cortex,
    Stem,
    Spine,
}

impl ObservabilitySubsystem {
    pub(crate) fn prefix(self) -> &'static str {
        match self {
            Self::AiGateway => "ai-gateway",
            Self::Cortex => "cortex",
            Self::Stem => "stem",
            Self::Spine => "spine",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FixtureBundle {
    pub schema_version: u32,
    pub subsystem: ObservabilitySubsystem,
    pub fixtures: Vec<FixtureCase>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FixtureCase {
    pub fixture_id: String,
    pub event: ContractEvent,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "family")]
pub enum ContractEvent {
    #[serde(rename = "ai-gateway.request")]
    AiGatewayRequest(AiGatewayRequestEvent),
    #[serde(rename = "ai-gateway.chat.turn")]
    AiGatewayChatTurn(AiGatewayChatTurnEvent),
    #[serde(rename = "ai-gateway.chat.thread")]
    AiGatewayChatThread(AiGatewayChatThreadEvent),
    #[serde(rename = "cortex.primary")]
    CortexPrimary(CortexOrganExecutionEvent),
    #[serde(rename = "cortex.sense-helper")]
    CortexSenseHelper(CortexOrganExecutionEvent),
    #[serde(rename = "cortex.goal-forest-helper")]
    CortexGoalForestHelper(CortexOrganExecutionEvent),
    #[serde(rename = "cortex.acts-helper")]
    CortexActsHelper(CortexOrganExecutionEvent),
    #[serde(rename = "cortex.goal-forest")]
    CortexGoalForest(CortexGoalForestEvent),
    #[serde(rename = "stem.tick")]
    StemTick(StemTickEvent),
    #[serde(rename = "stem.afferent")]
    StemAfferent(StemAfferentEvent),
    #[serde(rename = "stem.efferent")]
    StemEfferent(StemEfferentEvent),
    #[serde(rename = "stem.proprioception")]
    StemProprioception(StemProprioceptionEvent),
    #[serde(rename = "stem.ns-catalog")]
    StemNsCatalog(StemNsCatalogEvent),
    #[serde(rename = "stem.afferent.rule")]
    StemAfferentRule(StemAfferentRuleEvent),
    #[serde(rename = "spine.adapter")]
    SpineAdapter(SpineAdapterEvent),
    #[serde(rename = "spine.endpoint")]
    SpineEndpoint(SpineEndpointEvent),
    #[serde(rename = "spine.sense")]
    SpineSense(SpineSenseEvent),
    #[serde(rename = "spine.act")]
    SpineAct(SpineActEvent),
}

impl ContractEvent {
    pub fn family(&self) -> &'static str {
        match self {
            Self::AiGatewayRequest(_) => "ai-gateway.request",
            Self::AiGatewayChatTurn(_) => "ai-gateway.chat.turn",
            Self::AiGatewayChatThread(_) => "ai-gateway.chat.thread",
            Self::CortexPrimary(_) => "cortex.primary",
            Self::CortexSenseHelper(_) => "cortex.sense-helper",
            Self::CortexGoalForestHelper(_) => "cortex.goal-forest-helper",
            Self::CortexActsHelper(_) => "cortex.acts-helper",
            Self::CortexGoalForest(_) => "cortex.goal-forest",
            Self::StemTick(_) => "stem.tick",
            Self::StemAfferent(_) => "stem.afferent",
            Self::StemEfferent(_) => "stem.efferent",
            Self::StemProprioception(_) => "stem.proprioception",
            Self::StemNsCatalog(_) => "stem.ns-catalog",
            Self::StemAfferentRule(_) => "stem.afferent.rule",
            Self::SpineAdapter(_) => "spine.adapter",
            Self::SpineEndpoint(_) => "spine.endpoint",
            Self::SpineSense(_) => "spine.sense",
            Self::SpineAct(_) => "spine.act",
        }
    }

    pub(crate) fn subsystem(&self) -> ObservabilitySubsystem {
        match self {
            Self::AiGatewayRequest(_)
            | Self::AiGatewayChatTurn(_)
            | Self::AiGatewayChatThread(_) => ObservabilitySubsystem::AiGateway,
            Self::CortexPrimary(_)
            | Self::CortexSenseHelper(_)
            | Self::CortexGoalForestHelper(_)
            | Self::CortexActsHelper(_)
            | Self::CortexGoalForest(_) => ObservabilitySubsystem::Cortex,
            Self::StemTick(_)
            | Self::StemAfferent(_)
            | Self::StemEfferent(_)
            | Self::StemProprioception(_)
            | Self::StemNsCatalog(_)
            | Self::StemAfferentRule(_) => ObservabilitySubsystem::Stem,
            Self::SpineAdapter(_)
            | Self::SpineEndpoint(_)
            | Self::SpineSense(_)
            | Self::SpineAct(_) => ObservabilitySubsystem::Spine,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiGatewayRequestEvent {
    pub run_id: String,
    pub timestamp: String,
    pub tick: u64,
    pub request_id: String,
    pub span_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organ_id: Option<String>,
    pub capability: String,
    pub backend_id: String,
    pub model: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attempt: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retryable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_request: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_response: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiGatewayChatTurnEvent {
    pub run_id: String,
    pub timestamp: String,
    pub tick: u64,
    pub thread_id: String,
    pub turn_id: u64,
    pub span_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organ_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    pub status: String,
    pub dispatch_payload: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub messages_when_committed: Option<Value>,
    pub metadata: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backend_metadata: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiGatewayChatThreadEvent {
    pub run_id: String,
    pub timestamp: String,
    pub tick: u64,
    pub thread_id: String,
    pub span_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organ_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    pub kind: String,
    pub messages: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_summaries: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_turn_ids: Option<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrganResponseStatus {
    Ok,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CortexOrganExecutionEvent {
    pub run_id: String,
    pub timestamp: String,
    pub tick: u64,
    pub request_id: String,
    pub span_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<String>,
    pub phase: String,
    pub status: OrganResponseStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_or_backend: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_payload: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_payload: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai_request_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_id: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CortexGoalForestEvent {
    pub run_id: String,
    pub timestamp: String,
    pub tick: u64,
    pub span_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<String>,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mutation_request: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mutation_result: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub persisted_revision: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reset_context_applied: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_turn_ids: Option<Vec<u64>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StemTickEvent {
    pub run_id: String,
    pub timestamp: String,
    pub tick: u64,
    pub span_id: String,
    pub status: String,
    pub tick_seq: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DispatchOutcomeClass {
    Acknowledged,
    Rejected,
    Lost,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StemAfferentEvent {
    pub run_id: String,
    pub timestamp: String,
    pub tick: u64,
    pub span_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<String>,
    pub kind: String,
    pub descriptor_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sense_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sense_payload: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queue_state: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matched_rule_ids: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StemEfferentEvent {
    pub run_id: String,
    pub timestamp: String,
    pub tick: u64,
    pub span_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<String>,
    pub kind: String,
    pub act_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub descriptor_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub act_payload: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queue_state: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuity_decision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_outcome: Option<DispatchOutcomeClass>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StemProprioceptionEvent {
    pub run_id: String,
    pub timestamp: String,
    pub tick: u64,
    pub span_id: String,
    pub kind: String,
    pub entries_or_keys: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DescriptorCatalogChangeMode {
    Snapshot,
    Update,
    Drop,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StemNsCatalogEvent {
    pub run_id: String,
    pub timestamp: String,
    pub tick: u64,
    pub span_id: String,
    pub catalog_version: String,
    pub change_mode: DescriptorCatalogChangeMode,
    pub accepted_entries_or_routes: Value,
    pub rejected_entries_or_routes: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catalog_snapshot: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StemAfferentRuleEvent {
    pub run_id: String,
    pub timestamp: String,
    pub tick: u64,
    pub span_id: String,
    pub kind: String,
    pub revision: u64,
    pub rule_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub removed: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterLifecycleState {
    Enabled,
    Disabled,
    Faulted,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpineAdapterEvent {
    pub run_id: String,
    pub timestamp: String,
    pub tick: u64,
    pub span_id: String,
    pub adapter_type: String,
    pub adapter_id: String,
    pub kind: AdapterLifecycleState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_or_error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndpointLifecycleTransition {
    Connected,
    Disconnected,
    Registered,
    Dropped,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpineEndpointEvent {
    pub run_id: String,
    pub timestamp: String,
    pub tick: u64,
    pub span_id: String,
    pub endpoint_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter_id: Option<String>,
    pub kind: EndpointLifecycleTransition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_or_session: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_summary: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_or_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpineSenseEvent {
    pub run_id: String,
    pub timestamp: String,
    pub tick: u64,
    pub span_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<String>,
    pub endpoint_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub descriptor_id: Option<String>,
    pub sense_id: String,
    pub kind: String,
    pub sense_payload: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpineActEvent {
    pub run_id: String,
    pub timestamp: String,
    pub tick: u64,
    pub span_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<String>,
    pub act_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub descriptor_id: Option<String>,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub act_payload: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<DispatchOutcomeClass>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_or_reference: Option<Value>,
}
