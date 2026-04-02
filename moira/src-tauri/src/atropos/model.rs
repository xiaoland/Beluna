use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SupervisionPhase {
    Idle,
    Waking,
    Running,
    Stopping,
    Terminated,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatus {
    pub phase: SupervisionPhase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executable_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_reason: Option<String>,
}
