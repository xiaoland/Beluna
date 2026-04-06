use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    process::Stdio,
};

use async_trait::async_trait;
use flate2::read::GzDecoder;
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue, USER_AGENT};
use sha2::{Digest, Sha256};
use tar::Archive;
use tokio::process::Command;

use super::{
    ClothoService,
    model::{
        ForgeLocalBuildRequest, InstallPublishedReleaseRequest, InstalledArtifactRecord,
        KnownLocalBuildRecord, KnownLocalBuildRegistration, LaunchTargetProvenance,
        LaunchTargetReadiness, LaunchTargetRef, LaunchTargetSummary, PreparedLaunchTarget,
        PublishedReleaseSummary,
    },
    service::{canonicalize_dir, canonicalize_file, validate_ref_id},
};

const RELEASE_OWNER: &str = "xiaoland";
const RELEASE_REPO: &str = "Beluna";
const RELEASE_CHECKSUM_ASSET_NAME: &str = "SHA256SUMS";
const SUPPORTED_RELEASE_TARGET: &str = "aarch64-apple-darwin";
const RELEASE_ARCHIVE_PREFIX: &str = "beluna-core-";
const RELEASE_ARCHIVE_SUFFIX: &str = ".tar.gz";
const RELEASE_BINARY_NAME: &str = "beluna";

#[derive(Debug, Clone)]
pub(super) struct RemoteRelease {
    tag_name: String,
    display_name: String,
    prerelease: bool,
    published_at: Option<String>,
    assets: Vec<RemoteReleaseAsset>,
}

#[derive(Debug, Clone)]
pub(super) struct RemoteReleaseAsset {
    name: String,
    download_url: String,
}

#[derive(Debug, Clone)]
struct PublishedReleaseCandidate {
    summary: PublishedReleaseSummary,
    archive_url: String,
    checksum_url: String,
}

#[async_trait]
pub(super) trait ReleaseProvider: Send + Sync {
    async fn list_releases(&self) -> Result<Vec<RemoteRelease>, String>;
    async fn download_text(&self, url: &str) -> Result<String, String>;
    async fn download_bytes(&self, url: &str) -> Result<Vec<u8>, String>;
}

pub(super) struct GitHubReleaseProvider {
    client: reqwest::Client,
}

impl GitHubReleaseProvider {
    pub(super) fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("Beluna-Moira-Clotho/0.1"),
        );
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("github release client should build");

        Self { client }
    }
}

#[async_trait]
impl ReleaseProvider for GitHubReleaseProvider {
    async fn list_releases(&self) -> Result<Vec<RemoteRelease>, String> {
        let url = format!(
            "https://api.github.com/repos/{RELEASE_OWNER}/{RELEASE_REPO}/releases?per_page=20"
        );
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|err| format!("failed to query GitHub Releases: {err}"))?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(format!(
                "GitHub Releases query failed with status {status}: {}",
                truncate_for_error(&body)
            ));
        }

        let payload = response
            .json::<Vec<GitHubReleasePayload>>()
            .await
            .map_err(|err| format!("failed to decode GitHub Releases payload: {err}"))?;

        Ok(payload
            .into_iter()
            .map(|release| RemoteRelease {
                display_name: release.name.unwrap_or_else(|| release.tag_name.clone()),
                tag_name: release.tag_name,
                prerelease: release.prerelease,
                published_at: release.published_at,
                assets: release
                    .assets
                    .into_iter()
                    .map(|asset| RemoteReleaseAsset {
                        name: asset.name,
                        download_url: asset.browser_download_url,
                    })
                    .collect(),
            })
            .collect())
    }

    async fn download_text(&self, url: &str) -> Result<String, String> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|err| format!("failed to download release text asset `{url}`: {err}"))?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(format!(
                "release text asset download failed with status {status}: {}",
                truncate_for_error(&body)
            ));
        }

        response
            .text()
            .await
            .map_err(|err| format!("failed to decode release text asset `{url}`: {err}"))
    }

    async fn download_bytes(&self, url: &str) -> Result<Vec<u8>, String> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|err| format!("failed to download release archive `{url}`: {err}"))?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(format!(
                "release archive download failed with status {status}: {}",
                truncate_for_error(&body)
            ));
        }

        response
            .bytes()
            .await
            .map(|bytes| bytes.to_vec())
            .map_err(|err| format!("failed to read release archive `{url}`: {err}"))
    }
}

impl ClothoService {
    pub fn register_known_local_build(
        &self,
        registration: KnownLocalBuildRegistration,
    ) -> Result<LaunchTargetRef, String> {
        let build_id = validate_ref_id("build", &registration.build_id)?;
        let executable_path = canonicalize_file("build executable", &registration.executable_path)?;
        let working_dir = match registration.working_dir.as_ref() {
            Some(path) => canonicalize_dir("build working directory", path)?,
            None => executable_path
                .parent()
                .map(Path::to_path_buf)
                .ok_or_else(|| {
                    format!(
                        "build executable `{}` does not have a parent directory",
                        executable_path.display()
                    )
                })?,
        };
        let source_dir = registration
            .source_dir
            .as_ref()
            .map(|path| canonicalize_dir("build source directory", path))
            .transpose()?;

        self.write_known_local_build_record(&KnownLocalBuildRecord {
            build_id: build_id.clone(),
            executable_path,
            working_dir,
            source_dir,
            provenance: LaunchTargetProvenance::Registered,
        })?;

        Ok(LaunchTargetRef::KnownLocalBuild { build_id })
    }

    pub async fn forge_local_build(
        &self,
        request: ForgeLocalBuildRequest,
    ) -> Result<LaunchTargetRef, String> {
        let build_id = validate_ref_id("build", &request.build_id)?;
        let requested_source_dir = canonicalize_dir("build source directory", &request.source_dir)?;
        let core_crate_root = resolve_core_crate_root(&requested_source_dir)?;
        let manifest_path = core_crate_root.join("Cargo.toml");

        let output = Command::new(self.cargo_bin())
            .arg("build")
            .arg("--manifest-path")
            .arg(&manifest_path)
            .arg("--bin")
            .arg(RELEASE_BINARY_NAME)
            .current_dir(&core_crate_root)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|err| {
                format!(
                    "failed to start cargo build for `{}`: {err}",
                    core_crate_root.display()
                )
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let detail = if !stderr.trim().is_empty() {
                stderr.trim().to_string()
            } else {
                stdout.trim().to_string()
            };
            return Err(format!(
                "cargo build failed for `{}`: {}",
                core_crate_root.display(),
                truncate_for_error(&detail)
            ));
        }

        let executable_path = canonicalize_file(
            "forged executable",
            &core_crate_root
                .join("target")
                .join("debug")
                .join(executable_file_name()),
        )?;

        self.write_known_local_build_record(&KnownLocalBuildRecord {
            build_id: build_id.clone(),
            executable_path,
            working_dir: core_crate_root,
            source_dir: Some(requested_source_dir),
            provenance: LaunchTargetProvenance::Forged,
        })?;

        Ok(LaunchTargetRef::KnownLocalBuild { build_id })
    }

    pub fn list_launch_targets(&self) -> Result<Vec<LaunchTargetSummary>, String> {
        let mut targets = Vec::new();
        targets.extend(self.list_known_local_build_targets()?);
        targets.extend(self.list_installed_artifact_targets()?);
        targets.sort_by(|left, right| left.label.cmp(&right.label));
        Ok(targets)
    }

    pub async fn list_published_releases(&self) -> Result<Vec<PublishedReleaseSummary>, String> {
        Ok(self
            .published_release_candidates(SUPPORTED_RELEASE_TARGET)
            .await?
            .into_iter()
            .map(|candidate| candidate.summary)
            .collect())
    }

    pub async fn install_published_release(
        &self,
        request: InstallPublishedReleaseRequest,
    ) -> Result<LaunchTargetRef, String> {
        let release_tag = normalize_release_tag(&request.release_tag)?;
        let rust_target_triple =
            validate_ref_id("rust target triple", &request.rust_target_triple)?;
        if rust_target_triple != SUPPORTED_RELEASE_TARGET {
            return Err(format!(
                "unsupported release target `{rust_target_triple}`; current supported target is `{SUPPORTED_RELEASE_TARGET}`"
            ));
        }

        let candidate = self
            .find_published_release_candidate(&release_tag, &rust_target_triple)
            .await?;
        let checksum_contents = self
            .release_provider()
            .download_text(&candidate.checksum_url)
            .await?;
        let archive_bytes = self
            .release_provider()
            .download_bytes(&candidate.archive_url)
            .await?;

        let expected_checksum =
            parse_checksum(&checksum_contents, &candidate.summary.archive_asset_name)?;
        let actual_checksum = sha256_hex(&archive_bytes);
        if !expected_checksum.eq_ignore_ascii_case(&actual_checksum) {
            return Err(format!(
                "checksum mismatch for `{}`: expected {expected_checksum}, got {actual_checksum}",
                candidate.summary.archive_asset_name
            ));
        }

        let cache_dir = self.paths().release_cache_dir(&release_tag);
        fs::create_dir_all(&cache_dir).map_err(|err| {
            format!(
                "failed to create release cache directory `{}`: {err}",
                cache_dir.display()
            )
        })?;
        fs::write(
            cache_dir.join(&candidate.summary.archive_asset_name),
            &archive_bytes,
        )
        .map_err(|err| {
            format!("failed to write cached release archive for `{release_tag}`: {err}")
        })?;
        fs::write(
            cache_dir.join(RELEASE_CHECKSUM_ASSET_NAME),
            checksum_contents.as_bytes(),
        )
        .map_err(|err| {
            format!("failed to write cached checksum file for `{release_tag}`: {err}")
        })?;

        let install_dir = self
            .paths()
            .installed_artifact_dir(&release_tag, &rust_target_triple);
        if install_dir.exists() {
            fs::remove_dir_all(&install_dir).map_err(|err| {
                format!(
                    "failed to clear install directory `{}` before reinstall: {err}",
                    install_dir.display()
                )
            })?;
        }
        fs::create_dir_all(&install_dir).map_err(|err| {
            format!(
                "failed to create install directory `{}`: {err}",
                install_dir.display()
            )
        })?;

        let executable_path =
            extract_release_executable(&archive_bytes, &install_dir, executable_file_name())?;

        let record = InstalledArtifactRecord {
            release_tag: release_tag.clone(),
            rust_target_triple: rust_target_triple.clone(),
            archive_asset_name: candidate.summary.archive_asset_name.clone(),
            checksum_asset_name: candidate.summary.checksum_asset_name.clone(),
            checksum_sha256: actual_checksum,
            install_dir: install_dir.clone(),
            executable_path,
        };
        self.write_installed_artifact_record(&record)?;

        Ok(LaunchTargetRef::InstalledArtifact {
            release_tag,
            rust_target_triple,
        })
    }

    pub(super) fn resolve_launch_target(
        &self,
        target: &LaunchTargetRef,
    ) -> Result<PreparedLaunchTarget, String> {
        match target {
            LaunchTargetRef::KnownLocalBuild { build_id } => {
                self.resolve_known_local_build(build_id)
            }
            LaunchTargetRef::InstalledArtifact {
                release_tag,
                rust_target_triple,
            } => self.resolve_installed_artifact(release_tag, rust_target_triple),
        }
    }

    fn resolve_known_local_build(&self, build_id: &str) -> Result<PreparedLaunchTarget, String> {
        let record = self.read_known_local_build_record(build_id)?;
        let build_id = validate_ref_id("build", &record.build_id)?;

        Ok(PreparedLaunchTarget {
            target_label: build_id,
            executable_path: canonicalize_file("build executable", &record.executable_path)?,
            working_dir: canonicalize_dir("build working directory", &record.working_dir)?,
            provenance: record.provenance,
        })
    }

    fn resolve_installed_artifact(
        &self,
        release_tag: &str,
        rust_target_triple: &str,
    ) -> Result<PreparedLaunchTarget, String> {
        let record = self.read_installed_artifact_record(release_tag, rust_target_triple)?;
        let release_tag = normalize_release_tag(&record.release_tag)?;
        let rust_target_triple = validate_ref_id("rust target triple", &record.rust_target_triple)?;

        Ok(PreparedLaunchTarget {
            target_label: installed_label(&release_tag, &rust_target_triple),
            executable_path: canonicalize_file("installed executable", &record.executable_path)?,
            working_dir: canonicalize_dir("install directory", &record.install_dir)?,
            provenance: LaunchTargetProvenance::Installed,
        })
    }

    fn list_known_local_build_targets(&self) -> Result<Vec<LaunchTargetSummary>, String> {
        let directory = self.paths().known_local_builds_dir();
        let entries = fs::read_dir(&directory).map_err(|err| {
            format!(
                "failed to read known local builds directory `{}`: {err}",
                directory.display()
            )
        })?;

        let mut targets = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|err| {
                format!(
                    "failed to read an entry from known local builds directory `{}`: {err}",
                    directory.display()
                )
            })?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }

            targets.push(self.summarize_known_local_manifest(&path));
        }

        Ok(targets)
    }

    fn list_installed_artifact_targets(&self) -> Result<Vec<LaunchTargetSummary>, String> {
        let directory = self.paths().installed_artifacts_dir();
        let release_dirs = fs::read_dir(&directory).map_err(|err| {
            format!(
                "failed to read installed artifacts directory `{}`: {err}",
                directory.display()
            )
        })?;

        let mut targets = Vec::new();
        for release_dir in release_dirs {
            let release_dir = release_dir.map_err(|err| {
                format!(
                    "failed to read an installed artifact release entry from `{}`: {err}",
                    directory.display()
                )
            })?;
            if !release_dir.path().is_dir() {
                continue;
            }

            let triple_dirs = fs::read_dir(release_dir.path()).map_err(|err| {
                format!(
                    "failed to read installed artifact target directory `{}`: {err}",
                    release_dir.path().display()
                )
            })?;
            for triple_dir in triple_dirs {
                let triple_dir = triple_dir.map_err(|err| {
                    format!(
                        "failed to read an installed artifact target entry from `{}`: {err}",
                        release_dir.path().display()
                    )
                })?;
                if !triple_dir.path().is_dir() {
                    continue;
                }

                targets.push(
                    self.summarize_installed_manifest(&triple_dir.path().join("manifest.json")),
                );
            }
        }

        Ok(targets)
    }

    fn summarize_known_local_manifest(&self, manifest_path: &Path) -> LaunchTargetSummary {
        match fs::read_to_string(manifest_path)
            .map_err(|err| {
                format!(
                    "failed to read manifest `{}`: {err}",
                    manifest_path.display()
                )
            })
            .and_then(|source| {
                serde_json::from_str::<KnownLocalBuildRecord>(&source).map_err(|err| {
                    format!(
                        "failed to parse known local build manifest `{}`: {err}",
                        manifest_path.display()
                    )
                })
            }) {
            Ok(record) => summarize_known_local_record(record),
            Err(issue) => {
                let label = manifest_path
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .map(str::to_string)
                    .unwrap_or_else(|| "unknown-build".to_string());
                LaunchTargetSummary {
                    target: LaunchTargetRef::KnownLocalBuild {
                        build_id: label.clone(),
                    },
                    label,
                    provenance: LaunchTargetProvenance::Registered,
                    readiness: LaunchTargetReadiness::Stale,
                    issue: Some(issue),
                    executable_path: None,
                    working_dir: None,
                    source_dir: None,
                    install_dir: None,
                    release_tag: None,
                    rust_target_triple: None,
                    checksum_verified: false,
                }
            }
        }
    }

    fn summarize_installed_manifest(&self, manifest_path: &Path) -> LaunchTargetSummary {
        match fs::read_to_string(manifest_path)
            .map_err(|err| {
                format!(
                    "failed to read manifest `{}`: {err}",
                    manifest_path.display()
                )
            })
            .and_then(|source| {
                serde_json::from_str::<InstalledArtifactRecord>(&source).map_err(|err| {
                    format!(
                        "failed to parse installed artifact manifest `{}`: {err}",
                        manifest_path.display()
                    )
                })
            }) {
            Ok(record) => summarize_installed_record(record),
            Err(issue) => LaunchTargetSummary {
                target: LaunchTargetRef::InstalledArtifact {
                    release_tag: "unknown-release".to_string(),
                    rust_target_triple: SUPPORTED_RELEASE_TARGET.to_string(),
                },
                label: "unknown release".to_string(),
                provenance: LaunchTargetProvenance::Installed,
                readiness: LaunchTargetReadiness::Stale,
                issue: Some(issue),
                executable_path: None,
                working_dir: None,
                source_dir: None,
                install_dir: None,
                release_tag: None,
                rust_target_triple: None,
                checksum_verified: false,
            },
        }
    }

    async fn published_release_candidates(
        &self,
        rust_target_triple: &str,
    ) -> Result<Vec<PublishedReleaseCandidate>, String> {
        let rust_target_triple = validate_ref_id("rust target triple", rust_target_triple)?;
        let archive_name = archive_asset_name(&rust_target_triple);

        let releases = self.release_provider().list_releases().await?;
        let mut candidates = Vec::new();
        for release in releases {
            let archive_asset = release
                .assets
                .iter()
                .find(|asset| asset.name == archive_name);
            let checksum_asset = release
                .assets
                .iter()
                .find(|asset| asset.name == RELEASE_CHECKSUM_ASSET_NAME);
            let (Some(archive_asset), Some(checksum_asset)) = (archive_asset, checksum_asset)
            else {
                continue;
            };

            let release_tag = normalize_release_tag(&release.tag_name)?;
            candidates.push(PublishedReleaseCandidate {
                summary: PublishedReleaseSummary {
                    release_tag: release_tag.clone(),
                    display_name: release.display_name,
                    rust_target_triple: rust_target_triple.clone(),
                    archive_asset_name: archive_asset.name.clone(),
                    checksum_asset_name: checksum_asset.name.clone(),
                    prerelease: release.prerelease,
                    published_at: release.published_at,
                    already_installed: self
                        .paths()
                        .installed_artifact_manifest_path(&release_tag, &rust_target_triple)
                        .is_file(),
                },
                archive_url: archive_asset.download_url.clone(),
                checksum_url: checksum_asset.download_url.clone(),
            });
        }

        Ok(candidates)
    }

    async fn find_published_release_candidate(
        &self,
        release_tag: &str,
        rust_target_triple: &str,
    ) -> Result<PublishedReleaseCandidate, String> {
        let candidates = self
            .published_release_candidates(rust_target_triple)
            .await?;
        candidates
            .into_iter()
            .find(|candidate| candidate.summary.release_tag == release_tag)
            .ok_or_else(|| {
                format!(
                    "no published release candidate found for tag `{release_tag}` and target `{rust_target_triple}`"
                )
            })
    }

    fn read_known_local_build_record(
        &self,
        build_id: &str,
    ) -> Result<KnownLocalBuildRecord, String> {
        let build_id = validate_ref_id("build", build_id)?;
        let manifest_path = self.known_local_build_manifest_path(&build_id);
        let source = fs::read_to_string(&manifest_path).map_err(|err| {
            format!(
                "failed to read known local build manifest `{}`: {err}",
                manifest_path.display()
            )
        })?;
        let record = serde_json::from_str::<KnownLocalBuildRecord>(&source).map_err(|err| {
            format!(
                "failed to parse known local build manifest `{}`: {err}",
                manifest_path.display()
            )
        })?;
        if record.build_id != build_id {
            return Err(format!(
                "known local build manifest `{}` contained mismatched build id `{}`",
                manifest_path.display(),
                record.build_id
            ));
        }

        Ok(record)
    }

    fn read_installed_artifact_record(
        &self,
        release_tag: &str,
        rust_target_triple: &str,
    ) -> Result<InstalledArtifactRecord, String> {
        let release_tag = normalize_release_tag(release_tag)?;
        let rust_target_triple = validate_ref_id("rust target triple", rust_target_triple)?;
        let manifest_path = self
            .paths()
            .installed_artifact_manifest_path(&release_tag, &rust_target_triple);
        let source = fs::read_to_string(&manifest_path).map_err(|err| {
            format!(
                "failed to read installed artifact manifest `{}`: {err}",
                manifest_path.display()
            )
        })?;
        serde_json::from_str::<InstalledArtifactRecord>(&source).map_err(|err| {
            format!(
                "failed to parse installed artifact manifest `{}`: {err}",
                manifest_path.display()
            )
        })
    }

    fn write_known_local_build_record(&self, record: &KnownLocalBuildRecord) -> Result<(), String> {
        let manifest_path = self.known_local_build_manifest_path(&record.build_id);
        fs::create_dir_all(self.paths().known_local_builds_dir()).map_err(|err| {
            format!(
                "failed to create known local builds directory `{}`: {err}",
                self.paths().known_local_builds_dir().display()
            )
        })?;

        let rendered = serde_json::to_string_pretty(record)
            .map_err(|err| format!("failed to render build manifest: {err}"))?;
        fs::write(&manifest_path, format!("{rendered}\n")).map_err(|err| {
            format!(
                "failed to write known local build manifest `{}`: {err}",
                manifest_path.display()
            )
        })
    }

    fn write_installed_artifact_record(
        &self,
        record: &InstalledArtifactRecord,
    ) -> Result<(), String> {
        let manifest_path = self
            .paths()
            .installed_artifact_manifest_path(&record.release_tag, &record.rust_target_triple);
        let install_dir = manifest_path
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| {
                format!(
                    "installed artifact manifest `{}` had no parent",
                    manifest_path.display()
                )
            })?;
        fs::create_dir_all(&install_dir).map_err(|err| {
            format!(
                "failed to create installed artifact directory `{}`: {err}",
                install_dir.display()
            )
        })?;

        let rendered = serde_json::to_string_pretty(record)
            .map_err(|err| format!("failed to render installed artifact manifest: {err}"))?;
        fs::write(&manifest_path, format!("{rendered}\n")).map_err(|err| {
            format!(
                "failed to write installed artifact manifest `{}`: {err}",
                manifest_path.display()
            )
        })
    }

    fn known_local_build_manifest_path(&self, build_id: &str) -> PathBuf {
        self.paths()
            .known_local_builds_dir()
            .join(format!("{build_id}.json"))
    }
}

fn summarize_known_local_record(record: KnownLocalBuildRecord) -> LaunchTargetSummary {
    let issue = validate_known_local_record(&record).err();
    let readiness = if issue.is_some() {
        LaunchTargetReadiness::Stale
    } else {
        LaunchTargetReadiness::Ready
    };

    LaunchTargetSummary {
        target: LaunchTargetRef::KnownLocalBuild {
            build_id: record.build_id.clone(),
        },
        label: record.build_id,
        provenance: record.provenance,
        readiness,
        issue,
        executable_path: Some(record.executable_path),
        working_dir: Some(record.working_dir),
        source_dir: record.source_dir,
        install_dir: None,
        release_tag: None,
        rust_target_triple: None,
        checksum_verified: false,
    }
}

fn summarize_installed_record(record: InstalledArtifactRecord) -> LaunchTargetSummary {
    let issue = validate_installed_record(&record).err();
    let checksum_verified = issue.is_none();
    let readiness = if issue.is_some() {
        LaunchTargetReadiness::Stale
    } else {
        LaunchTargetReadiness::Ready
    };

    LaunchTargetSummary {
        target: LaunchTargetRef::InstalledArtifact {
            release_tag: record.release_tag.clone(),
            rust_target_triple: record.rust_target_triple.clone(),
        },
        label: installed_label(&record.release_tag, &record.rust_target_triple),
        provenance: LaunchTargetProvenance::Installed,
        readiness,
        issue,
        executable_path: Some(record.executable_path),
        working_dir: Some(record.install_dir.clone()),
        source_dir: None,
        install_dir: Some(record.install_dir),
        release_tag: Some(record.release_tag),
        rust_target_triple: Some(record.rust_target_triple),
        checksum_verified,
    }
}

fn validate_known_local_record(record: &KnownLocalBuildRecord) -> Result<(), String> {
    canonicalize_file("build executable", &record.executable_path)?;
    canonicalize_dir("build working directory", &record.working_dir)?;
    if let Some(source_dir) = record.source_dir.as_ref() {
        canonicalize_dir("build source directory", source_dir)?;
    }
    Ok(())
}

fn validate_installed_record(record: &InstalledArtifactRecord) -> Result<(), String> {
    canonicalize_dir("install directory", &record.install_dir)?;
    canonicalize_file("installed executable", &record.executable_path)?;
    Ok(())
}

fn resolve_core_crate_root(source_dir: &Path) -> Result<PathBuf, String> {
    let source_dir = canonicalize_dir("build source directory", source_dir)?;
    let repo_core = source_dir.join("core").join("Cargo.toml");
    if repo_core.is_file() {
        return canonicalize_dir("Beluna core crate root", &source_dir.join("core"));
    }

    let crate_manifest = source_dir.join("Cargo.toml");
    if crate_manifest.is_file() {
        return Ok(source_dir);
    }

    Err(format!(
        "source directory `{}` is neither a Beluna repo root nor a `core/` crate root",
        source_dir.display()
    ))
}

fn normalize_release_tag(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("release tag must not be empty".to_string());
    }
    if trimmed != value {
        return Err(format!(
            "release tag `{value}` must not contain leading or trailing whitespace"
        ));
    }

    Ok(trimmed.to_string())
}

fn archive_asset_name(rust_target_triple: &str) -> String {
    format!("{RELEASE_ARCHIVE_PREFIX}{rust_target_triple}{RELEASE_ARCHIVE_SUFFIX}")
}

fn installed_label(release_tag: &str, rust_target_triple: &str) -> String {
    format!("{release_tag} / {rust_target_triple}")
}

fn executable_file_name() -> &'static str {
    if cfg!(windows) {
        "beluna.exe"
    } else {
        RELEASE_BINARY_NAME
    }
}

fn parse_checksum(contents: &str, asset_name: &str) -> Result<String, String> {
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let Some((checksum, file_name)) = split_checksum_line(trimmed) else {
            continue;
        };
        if file_name == asset_name {
            return Ok(checksum.to_string());
        }
    }

    Err(format!(
        "checksum file did not contain an entry for `{asset_name}`"
    ))
}

fn split_checksum_line(line: &str) -> Option<(&str, &str)> {
    let mut parts = line.split_whitespace();
    let checksum = parts.next()?;
    let file_name = parts.next()?;
    Some((checksum, file_name.trim_start_matches('*')))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for value in digest {
        hex.push_str(&format!("{value:02x}"));
    }
    hex
}

fn extract_release_executable(
    archive_bytes: &[u8],
    install_dir: &Path,
    executable_name: &str,
) -> Result<PathBuf, String> {
    let mut archive = Archive::new(GzDecoder::new(Cursor::new(archive_bytes)));
    let entries = archive
        .entries()
        .map_err(|err| format!("failed to inspect release archive: {err}"))?;

    for entry in entries {
        let mut entry =
            entry.map_err(|err| format!("failed to read release archive entry: {err}"))?;
        let path = entry
            .path()
            .map_err(|err| format!("failed to resolve release archive entry path: {err}"))?;
        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if file_name != executable_name {
            continue;
        }

        let executable_path = install_dir.join(executable_name);
        entry.unpack(&executable_path).map_err(|err| {
            format!(
                "failed to extract `{executable_name}` into `{}`: {err}",
                install_dir.display()
            )
        })?;
        return canonicalize_file("installed executable", &executable_path);
    }

    Err(format!(
        "release archive did not contain `{executable_name}`"
    ))
}

fn truncate_for_error(value: &str) -> String {
    let trimmed = value.trim();
    const MAX_LEN: usize = 240;
    if trimmed.len() <= MAX_LEN {
        trimmed.to_string()
    } else {
        format!("{}...", &trimmed[..MAX_LEN])
    }
}

#[derive(Debug, serde::Deserialize)]
struct GitHubReleasePayload {
    tag_name: String,
    name: Option<String>,
    prerelease: bool,
    published_at: Option<String>,
    assets: Vec<GitHubReleaseAssetPayload>,
}

#[derive(Debug, serde::Deserialize)]
struct GitHubReleaseAssetPayload {
    name: String,
    browser_download_url: String,
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        fs,
        path::{Path, PathBuf},
        sync::Arc,
    };

    use flate2::{Compression, write::GzEncoder};
    use tar::{Builder, Header};
    use uuid::Uuid;

    use crate::app::state::AppPaths;

    use super::*;

    #[tokio::test]
    async fn forge_local_build_accepts_repo_root_and_prepares_launch_target() {
        let sandbox = TestSandbox::new();
        let repo_root = sandbox.create_dir("workspace/beluna");
        sandbox.create_file_with_contents(
            "workspace/beluna/core/Cargo.toml",
            "[package]\nname = \"beluna\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        );
        let cargo_bin = sandbox.create_fake_cargo();

        let paths = AppPaths::from_root(sandbox.root.join("app-data"));
        paths.ensure_dirs().expect("app paths should initialize");
        let service = ClothoService::with_dependencies(
            paths,
            cargo_bin,
            Arc::new(FixtureReleaseProvider::empty()),
        );

        let target = service
            .forge_local_build(ForgeLocalBuildRequest {
                build_id: "dev-core".to_string(),
                source_dir: repo_root.clone(),
            })
            .await
            .expect("forge should succeed");

        assert_eq!(
            target,
            LaunchTargetRef::KnownLocalBuild {
                build_id: "dev-core".to_string(),
            }
        );

        let prepared = service
            .resolve_launch_target(&target)
            .expect("forged target should resolve");
        assert_eq!(prepared.target_label, "dev-core");
        assert_eq!(prepared.provenance, LaunchTargetProvenance::Forged);
        assert_eq!(
            prepared.working_dir,
            fs::canonicalize(repo_root.join("core")).unwrap()
        );
        assert!(prepared.executable_path.is_file());

        let listed = service
            .list_launch_targets()
            .expect("launch targets should list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].readiness, LaunchTargetReadiness::Ready);
        assert_eq!(listed[0].provenance, LaunchTargetProvenance::Forged);
        assert_eq!(
            listed[0].source_dir,
            Some(fs::canonicalize(repo_root).unwrap())
        );
    }

    #[tokio::test]
    async fn install_published_release_creates_isolated_manifest_and_resolves() {
        let sandbox = TestSandbox::new();
        let paths = AppPaths::from_root(sandbox.root.join("app-data"));
        paths.ensure_dirs().expect("app paths should initialize");

        let archive_name = archive_asset_name(SUPPORTED_RELEASE_TARGET);
        let archive_bytes = build_release_archive(vec![(
            format!("bin/{RELEASE_BINARY_NAME}"),
            b"#!/bin/sh\nexit 0\n".to_vec(),
        )]);
        let checksum = sha256_hex(&archive_bytes);
        let provider = FixtureReleaseProvider::with_release(
            "v0.1.0",
            &archive_name,
            &archive_bytes,
            &format!("{checksum}  {archive_name}\n"),
        );
        let service = ClothoService::with_dependencies(
            paths.clone(),
            PathBuf::from("cargo"),
            Arc::new(provider),
        );

        let target = service
            .install_published_release(InstallPublishedReleaseRequest {
                release_tag: "v0.1.0".to_string(),
                rust_target_triple: SUPPORTED_RELEASE_TARGET.to_string(),
            })
            .await
            .expect("install should succeed");

        let prepared = service
            .resolve_launch_target(&target)
            .expect("installed target should resolve");
        assert_eq!(prepared.provenance, LaunchTargetProvenance::Installed);
        assert!(prepared.executable_path.is_file());
        assert_eq!(prepared.target_label, "v0.1.0 / aarch64-apple-darwin");

        let manifest_path =
            paths.installed_artifact_manifest_path("v0.1.0", SUPPORTED_RELEASE_TARGET);
        assert!(manifest_path.is_file());
        assert!(
            paths
                .release_cache_dir("v0.1.0")
                .join(RELEASE_CHECKSUM_ASSET_NAME)
                .is_file()
        );

        let listed = service
            .list_launch_targets()
            .expect("launch targets should list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].checksum_verified, true);
    }

    #[tokio::test]
    async fn install_published_release_rejects_checksum_mismatch() {
        let sandbox = TestSandbox::new();
        let paths = AppPaths::from_root(sandbox.root.join("app-data"));
        paths.ensure_dirs().expect("app paths should initialize");

        let archive_name = archive_asset_name(SUPPORTED_RELEASE_TARGET);
        let archive_bytes = build_release_archive(vec![(
            RELEASE_BINARY_NAME.to_string(),
            b"#!/bin/sh\nexit 0\n".to_vec(),
        )]);
        let provider = FixtureReleaseProvider::with_release(
            "v0.1.1",
            &archive_name,
            &archive_bytes,
            &format!("{}  {archive_name}\n", "deadbeef"),
        );
        let service =
            ClothoService::with_dependencies(paths, PathBuf::from("cargo"), Arc::new(provider));

        let err = service
            .install_published_release(InstallPublishedReleaseRequest {
                release_tag: "v0.1.1".to_string(),
                rust_target_triple: SUPPORTED_RELEASE_TARGET.to_string(),
            })
            .await
            .expect_err("checksum mismatch should fail");

        assert!(err.contains("checksum mismatch"));
    }

    #[tokio::test]
    async fn install_published_release_rejects_archive_without_executable() {
        let sandbox = TestSandbox::new();
        let paths = AppPaths::from_root(sandbox.root.join("app-data"));
        paths.ensure_dirs().expect("app paths should initialize");

        let archive_name = archive_asset_name(SUPPORTED_RELEASE_TARGET);
        let archive_bytes = build_release_archive(vec![(
            "README.txt".to_string(),
            b"missing binary\n".to_vec(),
        )]);
        let checksum = sha256_hex(&archive_bytes);
        let provider = FixtureReleaseProvider::with_release(
            "v0.1.2",
            &archive_name,
            &archive_bytes,
            &format!("{checksum}  {archive_name}\n"),
        );
        let service =
            ClothoService::with_dependencies(paths, PathBuf::from("cargo"), Arc::new(provider));

        let err = service
            .install_published_release(InstallPublishedReleaseRequest {
                release_tag: "v0.1.2".to_string(),
                rust_target_triple: SUPPORTED_RELEASE_TARGET.to_string(),
            })
            .await
            .expect_err("missing executable should fail");

        assert!(err.contains("did not contain"));
    }

    #[test]
    fn register_and_prepare_known_local_build_round_trip() {
        let sandbox = TestSandbox::new();
        let paths = AppPaths::from_root(sandbox.root.join("app-data"));
        paths.ensure_dirs().expect("app paths should initialize");

        let executable_path = sandbox.create_file("fixtures/builds/dev-core/bin/beluna");
        let source_dir = sandbox.create_dir("fixtures/builds/dev-core/src");
        let profile_path = sandbox.create_file_with_contents(
            "app-data/profiles/default.jsonc",
            r#"
            {
              core_config: {
                logging: {
                  dir: "./logs"
                }
              }
            }
            "#,
        );
        assert!(profile_path.exists());

        let service = ClothoService::with_dependencies(
            paths,
            PathBuf::from("cargo"),
            Arc::new(FixtureReleaseProvider::empty()),
        );
        let target = service
            .register_known_local_build(KnownLocalBuildRegistration {
                build_id: "dev-core".to_string(),
                executable_path: executable_path.clone(),
                working_dir: None,
                source_dir: Some(source_dir.clone()),
            })
            .expect("known local build should register");
        let prepared = service
            .prepare_wake_input(&super::super::model::WakeInputRequest {
                target,
                profile: Some(super::super::model::ProfileRef {
                    profile_id: "default".to_string(),
                }),
            })
            .expect("wake input should resolve");

        assert_eq!(prepared.target.target_label, "dev-core");
        assert_eq!(
            prepared.target.executable_path,
            fs::canonicalize(executable_path).expect("executable path should resolve")
        );
        assert_eq!(
            prepared.profile_path,
            Some(fs::canonicalize(profile_path).expect("profile path should resolve"))
        );
    }

    struct TestSandbox {
        root: PathBuf,
    }

    impl TestSandbox {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!("moira-clotho-test-{}", Uuid::now_v7()));
            fs::create_dir_all(&root).expect("sandbox should create");
            Self { root }
        }

        fn create_dir(&self, relative: &str) -> PathBuf {
            let path = self.root.join(relative);
            fs::create_dir_all(&path).expect("directory should create");
            path
        }

        fn create_file(&self, relative: &str) -> PathBuf {
            self.create_file_with_contents(relative, "#!/bin/sh\nexit 0\n")
        }

        fn create_file_with_contents(&self, relative: &str, contents: impl AsRef<[u8]>) -> PathBuf {
            let path = self.root.join(relative);
            let parent = path.parent().expect("file should have parent");
            fs::create_dir_all(parent).expect("file parent should create");
            fs::write(&path, contents).expect("file should write");
            path
        }

        fn create_fake_cargo(&self) -> PathBuf {
            let path = self.root.join("bin/fake-cargo.sh");
            let parent = path.parent().expect("fake cargo should have parent");
            fs::create_dir_all(parent).expect("fake cargo parent should create");
            let script = format!(
                "#!/bin/sh\nmkdir -p \"$PWD/target/debug\"\nprintf '#!/bin/sh\\nexit 0\\n' > \"$PWD/target/debug/{}\"\nchmod +x \"$PWD/target/debug/{}\"\n",
                executable_file_name(),
                executable_file_name(),
            );
            fs::write(&path, script).expect("fake cargo should write");
            make_executable(&path);
            path
        }
    }

    impl Drop for TestSandbox {
        fn drop(&mut self) {
            let _ = remove_dir_all_if_exists(&self.root);
        }
    }

    struct FixtureReleaseProvider {
        releases: Vec<RemoteRelease>,
        texts: HashMap<String, String>,
        binaries: HashMap<String, Vec<u8>>,
    }

    impl FixtureReleaseProvider {
        fn empty() -> Self {
            Self {
                releases: Vec::new(),
                texts: HashMap::new(),
                binaries: HashMap::new(),
            }
        }

        fn with_release(
            release_tag: &str,
            archive_name: &str,
            archive_bytes: &[u8],
            checksum_contents: &str,
        ) -> Self {
            let archive_url = format!("fixture://archive/{release_tag}/{archive_name}");
            let checksum_url =
                format!("fixture://checksum/{release_tag}/{RELEASE_CHECKSUM_ASSET_NAME}");
            let mut provider = Self::empty();
            provider.releases.push(RemoteRelease {
                tag_name: release_tag.to_string(),
                display_name: release_tag.to_string(),
                prerelease: true,
                published_at: Some("2026-04-04T00:00:00Z".to_string()),
                assets: vec![
                    RemoteReleaseAsset {
                        name: archive_name.to_string(),
                        download_url: archive_url.clone(),
                    },
                    RemoteReleaseAsset {
                        name: RELEASE_CHECKSUM_ASSET_NAME.to_string(),
                        download_url: checksum_url.clone(),
                    },
                ],
            });
            provider
                .texts
                .insert(checksum_url, checksum_contents.to_string());
            provider
                .binaries
                .insert(archive_url, archive_bytes.to_vec());
            provider
        }
    }

    #[async_trait]
    impl ReleaseProvider for FixtureReleaseProvider {
        async fn list_releases(&self) -> Result<Vec<RemoteRelease>, String> {
            Ok(self.releases.clone())
        }

        async fn download_text(&self, url: &str) -> Result<String, String> {
            self.texts
                .get(url)
                .cloned()
                .ok_or_else(|| format!("missing fixture text for `{url}`"))
        }

        async fn download_bytes(&self, url: &str) -> Result<Vec<u8>, String> {
            self.binaries
                .get(url)
                .cloned()
                .ok_or_else(|| format!("missing fixture bytes for `{url}`"))
        }
    }

    fn build_release_archive(entries: Vec<(String, Vec<u8>)>) -> Vec<u8> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        {
            let mut builder = Builder::new(&mut encoder);
            for (path, contents) in entries {
                let mut header = Header::new_gnu();
                header.set_size(contents.len() as u64);
                header.set_mode(0o755);
                header.set_cksum();
                builder
                    .append_data(&mut header, path, Cursor::new(contents))
                    .expect("archive entry should append");
            }
            builder.finish().expect("archive should finish");
        }
        encoder.finish().expect("archive bytes should finish")
    }

    fn remove_dir_all_if_exists(path: &Path) -> std::io::Result<()> {
        if path.exists() {
            fs::remove_dir_all(path)?;
        }
        Ok(())
    }

    fn make_executable(path: &Path) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mut permissions = fs::metadata(path).expect("file should stat").permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(path, permissions).expect("permissions should update");
        }
    }
}
