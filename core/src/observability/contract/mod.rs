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
    #[serde(rename = "ai-gateway.turn")]
    AiGatewayTurn(AiGatewayTurnEvent),
    #[serde(rename = "ai-gateway.thread")]
    AiGatewayThread(AiGatewayThreadEvent),
    #[serde(rename = "cortex.tick")]
    CortexTick(CortexTickEvent),
    #[serde(rename = "cortex.organ")]
    CortexOrgan(CortexOrganEvent),
    #[serde(rename = "cortex.goal-forest")]
    CortexGoalForest(CortexGoalForestEvent),
    #[serde(rename = "stem.tick")]
    StemTick(StemTickEvent),
    #[serde(rename = "stem.signal")]
    StemSignal(StemSignalEvent),
    #[serde(rename = "stem.dispatch")]
    StemDispatch(StemDispatchEvent),
    #[serde(rename = "stem.proprioception")]
    StemProprioception(StemProprioceptionEvent),
    #[serde(rename = "stem.descriptor.catalog")]
    StemDescriptorCatalog(StemDescriptorCatalogEvent),
    #[serde(rename = "stem.afferent.rule")]
    StemAfferentRule(StemAfferentRuleEvent),
    #[serde(rename = "spine.adapter")]
    SpineAdapter(SpineAdapterEvent),
    #[serde(rename = "spine.endpoint")]
    SpineEndpoint(SpineEndpointEvent),
    #[serde(rename = "spine.dispatch")]
    SpineDispatch(SpineDispatchEvent),
}

impl ContractEvent {
    pub fn family(&self) -> &'static str {
        match self {
            Self::AiGatewayRequest(_) => "ai-gateway.request",
            Self::AiGatewayTurn(_) => "ai-gateway.turn",
            Self::AiGatewayThread(_) => "ai-gateway.thread",
            Self::CortexTick(_) => "cortex.tick",
            Self::CortexOrgan(_) => "cortex.organ",
            Self::CortexGoalForest(_) => "cortex.goal-forest",
            Self::StemTick(_) => "stem.tick",
            Self::StemSignal(_) => "stem.signal",
            Self::StemDispatch(_) => "stem.dispatch",
            Self::StemProprioception(_) => "stem.proprioception",
            Self::StemDescriptorCatalog(_) => "stem.descriptor.catalog",
            Self::StemAfferentRule(_) => "stem.afferent.rule",
            Self::SpineAdapter(_) => "spine.adapter",
            Self::SpineEndpoint(_) => "spine.endpoint",
            Self::SpineDispatch(_) => "spine.dispatch",
        }
    }

    pub(crate) fn subsystem(&self) -> ObservabilitySubsystem {
        match self {
            Self::AiGatewayRequest(_) | Self::AiGatewayTurn(_) | Self::AiGatewayThread(_) => {
                ObservabilitySubsystem::AiGateway
            }
            Self::CortexTick(_) | Self::CortexOrgan(_) | Self::CortexGoalForest(_) => {
                ObservabilitySubsystem::Cortex
            }
            Self::StemTick(_)
            | Self::StemSignal(_)
            | Self::StemDispatch(_)
            | Self::StemProprioception(_)
            | Self::StemDescriptorCatalog(_)
            | Self::StemAfferentRule(_) => ObservabilitySubsystem::Stem,
            Self::SpineAdapter(_) | Self::SpineEndpoint(_) | Self::SpineDispatch(_) => {
                ObservabilitySubsystem::Spine
            }
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
    pub parent_span_id_when_present: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organ_id_when_present: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_id_when_present: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_id_when_present: Option<u64>,
    pub backend_id: String,
    pub model: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attempt_when_present: Option<u32>,
    pub input_payload: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_tools_when_present: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limits_when_present: Option<Value>,
    pub enable_thinking: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_request_when_present: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_response_when_present: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage_when_present: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_when_present: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiGatewayTurnEvent {
    pub run_id: String,
    pub timestamp: String,
    pub tick: u64,
    pub thread_id: String,
    pub turn_id: u64,
    pub span_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_span_id_when_present: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organ_id_when_present: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id_when_present: Option<String>,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub messages_when_committed: Option<Value>,
    pub metadata: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finish_reason_when_present: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage_when_present: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backend_metadata_when_present: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_when_present: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiGatewayThreadEvent {
    pub run_id: String,
    pub timestamp: String,
    pub tick: u64,
    pub thread_id: String,
    pub span_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_span_id_when_present: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organ_id_when_present: Option<String>,
    pub kind: String,
    pub messages: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_summaries_when_present: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_turn_ids_when_present: Option<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrganResponseStatus {
    Ok,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CortexTickEvent {
    pub run_id: String,
    pub timestamp: String,
    pub tick: u64,
    pub span_id: String,
    pub kind_or_status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tick_seq_when_present: Option<u64>,
    pub drained_senses: Value,
    pub physical_state_snapshot: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub control_gate_state_when_present: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acts_payload_or_summary_when_present: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal_forest_snapshot_ref_or_payload_when_present: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_when_present: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CortexOrganEvent {
    pub run_id: String,
    pub timestamp: String,
    pub tick: u64,
    pub organ_id: String,
    pub request_id: String,
    pub span_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_span_id_when_present: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_or_backend_when_present: Option<String>,
    pub phase: String,
    pub status: OrganResponseStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_payload_when_present: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_payload_when_present: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_when_present: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai_gateway_request_id_when_present: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_id_when_present: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_id_when_present: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CortexGoalForestEvent {
    pub run_id: String,
    pub timestamp: String,
    pub tick: u64,
    pub span_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_span_id_when_present: Option<String>,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot_when_present: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patch_request_when_present: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patch_result_when_present: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cognition_persisted_revision_when_present: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reset_context_applied_when_present: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_turn_ids_when_present: Option<Vec<u64>>,
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
pub enum SignalDirection {
    Afferent,
    Efferent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitionKind {
    Enqueue,
    Defer,
    Release,
    Dispatch,
    Result,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StemSignalEvent {
    pub run_id: String,
    pub timestamp: String,
    pub tick: u64,
    pub span_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_span_id_when_present: Option<String>,
    pub direction: SignalDirection,
    pub transition_kind: TransitionKind,
    pub descriptor_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint_id_when_present: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sense_id_when_present: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub act_id_when_present: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sense_payload_when_present: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub act_payload_when_present: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight_when_present: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queue_or_deferred_state_when_present: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matched_rule_ids_when_present: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_when_present: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DispatchOutcomeClass {
    Acknowledged,
    Rejected,
    Lost,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StemDispatchEvent {
    pub run_id: String,
    pub timestamp: String,
    pub tick: u64,
    pub span_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_span_id_when_present: Option<String>,
    pub act_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub descriptor_id_when_present: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint_id_when_present: Option<String>,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub act_payload_when_present: Option<Value>,
    pub queue_or_flow_summary: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuity_decision_when_present: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_outcome_when_present: Option<DispatchOutcomeClass>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_or_reference_when_present: Option<Value>,
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
pub struct StemDescriptorCatalogEvent {
    pub run_id: String,
    pub timestamp: String,
    pub tick: u64,
    pub span_id: String,
    pub catalog_version: String,
    pub change_mode: DescriptorCatalogChangeMode,
    pub accepted_entries_or_routes: Value,
    pub rejected_entries_or_routes: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catalog_snapshot_when_required: Option<Value>,
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
    pub rule_when_present: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub removed_when_present: Option<bool>,
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
    pub kind_or_state: AdapterLifecycleState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_or_error_when_present: Option<String>,
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
    pub kind_or_transition: EndpointLifecycleTransition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter_id_when_present: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_or_session_when_present: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_summary_when_present: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_or_error_when_present: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpineDispatchEvent {
    pub run_id: String,
    pub timestamp: String,
    pub tick: u64,
    pub span_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_span_id_when_present: Option<String>,
    pub act_id: String,
    pub endpoint_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub descriptor_id_when_present: Option<String>,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding_kind_when_present: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_id_when_present: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome_when_present: Option<DispatchOutcomeClass>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_code_when_present: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_id_when_present: Option<String>,
}
