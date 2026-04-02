use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnownLocalBuildRef {
    pub build_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRef {
    pub profile_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDocumentSummary {
    pub profile_id: String,
    pub profile_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDocument {
    pub profile_id: String,
    pub profile_path: PathBuf,
    pub contents: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveProfileDocumentRequest {
    pub profile_id: String,
    pub contents: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WakeInputRequest {
    pub build: KnownLocalBuildRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<ProfileRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnownLocalBuildRegistration {
    pub build_id: String,
    pub executable_path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreparedKnownLocalBuild {
    pub build_id: String,
    pub executable_path: PathBuf,
    pub working_dir: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreparedWakeInput {
    pub build: PreparedKnownLocalBuild,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KnownLocalBuildRecord {
    pub build_id: String,
    pub executable_path: PathBuf,
    pub working_dir: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_dir: Option<PathBuf>,
}
