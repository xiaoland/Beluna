use serde::Serialize;

use crate::{atropos::model::RuntimeStatus, lachesis::model::ReceiverStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MoiraRuntimeLifecycle {
    Opening,
    Ready,
    Degraded,
    Closing,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MoiraResourceKind {
    Directory,
    TelemetryStore,
    OtlpReceiver,
    CoreSupervisor,
    PlatformAdapter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MoiraResourceState {
    Available,
    Claiming,
    Claimed,
    Degraded,
    Conflict,
    Faulted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoiraResourceStatus {
    pub kind: MoiraResourceKind,
    pub state: MoiraResourceState,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoiraRuntimeStatus {
    pub lifecycle: MoiraRuntimeLifecycle,
    pub resources: Vec<MoiraResourceStatus>,
    pub receiver: ReceiverStatus,
    pub core: RuntimeStatus,
}
