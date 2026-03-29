mod validate;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use validate::ContractValidationError;

pub const FIXTURE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservabilitySubsystem {
    Cortex,
    Stem,
    Spine,
}

impl ObservabilitySubsystem {
    pub(crate) fn prefix(self) -> &'static str {
        match self {
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
    #[serde(rename = "cortex.tick")]
    CortexTick(CortexTickEvent),
    #[serde(rename = "cortex.organ.request")]
    CortexOrganRequest(CortexOrganRequestEvent),
    #[serde(rename = "cortex.organ.response")]
    CortexOrganResponse(CortexOrganResponseEvent),
    #[serde(rename = "cortex.goal_forest.snapshot")]
    CortexGoalForestSnapshot(CortexGoalForestSnapshotEvent),
    #[serde(rename = "stem.signal.transition")]
    StemSignalTransition(StemSignalTransitionEvent),
    #[serde(rename = "stem.dispatch.transition")]
    StemDispatchTransition(StemDispatchTransitionEvent),
    #[serde(rename = "stem.descriptor.catalog")]
    StemDescriptorCatalog(StemDescriptorCatalogEvent),
    #[serde(rename = "spine.adapter.lifecycle")]
    SpineAdapterLifecycle(SpineAdapterLifecycleEvent),
    #[serde(rename = "spine.endpoint.lifecycle")]
    SpineEndpointLifecycle(SpineEndpointLifecycleEvent),
    #[serde(rename = "spine.dispatch.outcome")]
    SpineDispatchOutcome(SpineDispatchOutcomeEvent),
}

impl ContractEvent {
    pub fn family(&self) -> &'static str {
        match self {
            Self::CortexTick(_) => "cortex.tick",
            Self::CortexOrganRequest(_) => "cortex.organ.request",
            Self::CortexOrganResponse(_) => "cortex.organ.response",
            Self::CortexGoalForestSnapshot(_) => "cortex.goal_forest.snapshot",
            Self::StemSignalTransition(_) => "stem.signal.transition",
            Self::StemDispatchTransition(_) => "stem.dispatch.transition",
            Self::StemDescriptorCatalog(_) => "stem.descriptor.catalog",
            Self::SpineAdapterLifecycle(_) => "spine.adapter.lifecycle",
            Self::SpineEndpointLifecycle(_) => "spine.endpoint.lifecycle",
            Self::SpineDispatchOutcome(_) => "spine.dispatch.outcome",
        }
    }

    pub(crate) fn subsystem(&self) -> ObservabilitySubsystem {
        match self {
            Self::CortexTick(_)
            | Self::CortexOrganRequest(_)
            | Self::CortexOrganResponse(_)
            | Self::CortexGoalForestSnapshot(_) => ObservabilitySubsystem::Cortex,
            Self::StemSignalTransition(_)
            | Self::StemDispatchTransition(_)
            | Self::StemDescriptorCatalog(_) => ObservabilitySubsystem::Stem,
            Self::SpineAdapterLifecycle(_)
            | Self::SpineEndpointLifecycle(_)
            | Self::SpineDispatchOutcome(_) => ObservabilitySubsystem::Spine,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CortexTickEvent {
    pub run_id: String,
    pub timestamp: String,
    pub tick: u64,
    pub trigger_summary: Value,
    pub senses_summary: Value,
    pub proprioception_snapshot_or_ref: Value,
    pub acts_summary: Value,
    pub goal_forest_ref: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CortexOrganRequestEvent {
    pub run_id: String,
    pub timestamp: String,
    pub tick: u64,
    pub stage: String,
    pub route_or_organ: String,
    pub request_id: String,
    pub input_summary: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrganResponseStatus {
    Ok,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CortexOrganResponseEvent {
    pub run_id: String,
    pub timestamp: String,
    pub tick: u64,
    pub stage: String,
    pub request_id: String,
    pub status: OrganResponseStatus,
    pub response_summary: Value,
    pub tool_summary: Value,
    pub act_summary: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_summary_when_present: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CortexGoalForestSnapshotEvent {
    pub run_id: String,
    pub timestamp: String,
    pub tick: u64,
    pub snapshot_summary: Value,
    pub snapshot_or_ref: Value,
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
pub struct StemSignalTransitionEvent {
    pub run_id: String,
    pub timestamp: String,
    pub direction: SignalDirection,
    pub transition_kind: TransitionKind,
    pub descriptor_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sense_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub act_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tick_when_known: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DispatchOutcomeClass {
    Acknowledged,
    Rejected,
    Lost,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StemDispatchTransitionEvent {
    pub run_id: String,
    pub timestamp: String,
    pub act_id: String,
    pub transition_kind: TransitionKind,
    pub queue_or_flow_summary: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tick_when_known: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_outcome_when_present: Option<DispatchOutcomeClass>,
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
    pub catalog_version: String,
    pub change_mode: DescriptorCatalogChangeMode,
    pub changed_descriptor_summary: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catalog_snapshot_when_required: Option<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterLifecycleState {
    Enabled,
    Disabled,
    Faulted,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpineAdapterLifecycleEvent {
    pub run_id: String,
    pub timestamp: String,
    pub adapter_type: String,
    pub adapter_id: String,
    pub state_transition: AdapterLifecycleState,
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
pub struct SpineEndpointLifecycleEvent {
    pub run_id: String,
    pub timestamp: String,
    pub endpoint_id: String,
    pub transition_kind: EndpointLifecycleTransition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_or_session_when_present: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_or_error_when_present: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpineDispatchOutcomeEvent {
    pub run_id: String,
    pub timestamp: String,
    pub act_id: String,
    pub binding_target: String,
    pub outcome: DispatchOutcomeClass,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub descriptor_id_when_present: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latency_ms_when_present: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tick_when_known: Option<u64>,
}
