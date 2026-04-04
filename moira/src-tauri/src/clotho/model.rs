use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRef {
    pub profile_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LaunchTargetRef {
    KnownLocalBuild {
        #[serde(rename = "buildId")]
        build_id: String,
    },
    InstalledArtifact {
        #[serde(rename = "releaseTag")]
        release_tag: String,
        #[serde(rename = "rustTargetTriple")]
        rust_target_triple: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LaunchTargetProvenance {
    Registered,
    Forged,
    Installed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LaunchTargetReadiness {
    Ready,
    Stale,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchTargetSummary {
    pub target: LaunchTargetRef,
    pub label: String,
    pub provenance: LaunchTargetProvenance,
    pub readiness: LaunchTargetReadiness,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executable_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_dir: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_dir: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rust_target_triple: Option<String>,
    pub checksum_verified: bool,
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WakeInputRequest {
    pub target: LaunchTargetRef,
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForgeLocalBuildRequest {
    pub build_id: String,
    pub source_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallPublishedReleaseRequest {
    pub release_tag: String,
    pub rust_target_triple: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishedReleaseSummary {
    pub release_tag: String,
    pub display_name: String,
    pub rust_target_triple: String,
    pub archive_asset_name: String,
    pub checksum_asset_name: String,
    pub prerelease: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,
    pub already_installed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreparedLaunchTarget {
    pub target_label: String,
    pub executable_path: PathBuf,
    pub working_dir: PathBuf,
    pub provenance: LaunchTargetProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreparedWakeInput {
    pub target: PreparedLaunchTarget,
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
    #[serde(default = "default_registered_provenance")]
    pub provenance: LaunchTargetProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InstalledArtifactRecord {
    pub release_tag: String,
    pub rust_target_triple: String,
    pub archive_asset_name: String,
    pub checksum_asset_name: String,
    pub checksum_sha256: String,
    pub install_dir: PathBuf,
    pub executable_path: PathBuf,
}

fn default_registered_provenance() -> LaunchTargetProvenance {
    LaunchTargetProvenance::Registered
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{LaunchTargetRef, ProfileRef, WakeInputRequest};

    #[test]
    fn launch_target_ref_serializes_known_local_build_fields_in_camel_case() {
        let value = serde_json::to_value(LaunchTargetRef::KnownLocalBuild {
            build_id: "dev-core".to_string(),
        })
        .expect("known local build should serialize");

        assert_eq!(
            value,
            json!({
                "kind": "knownLocalBuild",
                "buildId": "dev-core",
            })
        );
    }

    #[test]
    fn launch_target_ref_serializes_installed_artifact_fields_in_camel_case() {
        let value = serde_json::to_value(LaunchTargetRef::InstalledArtifact {
            release_tag: "v0.1.0".to_string(),
            rust_target_triple: "aarch64-apple-darwin".to_string(),
        })
        .expect("installed artifact should serialize");

        assert_eq!(
            value,
            json!({
                "kind": "installedArtifact",
                "releaseTag": "v0.1.0",
                "rustTargetTriple": "aarch64-apple-darwin",
            })
        );
    }

    #[test]
    fn wake_input_request_deserializes_known_local_build_from_camel_case_payload() {
        let request = serde_json::from_value::<WakeInputRequest>(json!({
            "target": {
                "kind": "knownLocalBuild",
                "buildId": "dev-core",
            },
            "profile": {
                "profileId": "default",
            },
        }))
        .expect("wake request should deserialize");

        assert_eq!(
            request,
            WakeInputRequest {
                target: LaunchTargetRef::KnownLocalBuild {
                    build_id: "dev-core".to_string(),
                },
                profile: Some(ProfileRef {
                    profile_id: "default".to_string(),
                }),
            }
        );
    }

    #[test]
    fn wake_input_request_deserializes_installed_artifact_from_camel_case_payload() {
        let request = serde_json::from_value::<WakeInputRequest>(json!({
            "target": {
                "kind": "installedArtifact",
                "releaseTag": "v0.1.0",
                "rustTargetTriple": "aarch64-apple-darwin",
            },
        }))
        .expect("wake request should deserialize");

        assert_eq!(
            request,
            WakeInputRequest {
                target: LaunchTargetRef::InstalledArtifact {
                    release_tag: "v0.1.0".to_string(),
                    rust_target_triple: "aarch64-apple-darwin".to_string(),
                },
                profile: None,
            }
        );
    }
}
