use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::app::state::AppPaths;

use super::{
    artifacts::{GitHubReleaseProvider, ReleaseProvider},
    model::{PreparedRuntimeWakeInput, PreparedWakeInput, WakeInputRequest},
    profile_runtime::prepare_profile_runtime,
};

pub struct ClothoService {
    paths: AppPaths,
    cargo_bin: PathBuf,
    release_provider: Arc<dyn ReleaseProvider>,
}

impl ClothoService {
    pub fn new(paths: AppPaths) -> Self {
        Self::with_dependencies(
            paths,
            PathBuf::from("cargo"),
            Arc::new(GitHubReleaseProvider::new()),
        )
    }

    pub(super) fn with_dependencies(
        paths: AppPaths,
        cargo_bin: PathBuf,
        release_provider: Arc<dyn ReleaseProvider>,
    ) -> Self {
        Self {
            paths,
            cargo_bin,
            release_provider,
        }
    }

    pub fn paths(&self) -> &AppPaths {
        &self.paths
    }

    pub(super) fn cargo_bin(&self) -> &Path {
        &self.cargo_bin
    }

    pub(super) fn release_provider(&self) -> &(dyn ReleaseProvider + Send + Sync) {
        self.release_provider.as_ref()
    }

    pub fn prepare_wake_input(
        &self,
        request: &WakeInputRequest,
    ) -> Result<PreparedWakeInput, String> {
        let prepared = self.prepare_runtime_wake_input(request)?;

        Ok(PreparedWakeInput {
            target: prepared.target,
            profile_path: prepared.profile_path,
        })
    }

    pub(crate) fn prepare_runtime_wake_input(
        &self,
        request: &WakeInputRequest,
    ) -> Result<PreparedRuntimeWakeInput, String> {
        let target = self.resolve_launch_target(&request.target)?;
        let profile_runtime = request
            .profile
            .as_ref()
            .map(|profile| {
                let profile_path = self.resolve_profile_path(&profile.profile_id)?;
                prepare_profile_runtime(&profile.profile_id, &profile_path)
            })
            .transpose()?;

        Ok(PreparedRuntimeWakeInput {
            target,
            profile_path: profile_runtime
                .as_ref()
                .map(|profile| profile.profile_path.clone()),
            config_path: profile_runtime
                .as_ref()
                .map(|profile| profile.config_path.clone()),
            environment_overrides: profile_runtime
                .map(|profile| profile.environment_overrides)
                .unwrap_or_default(),
        })
    }

    fn resolve_profile_path(&self, profile_id: &str) -> Result<PathBuf, String> {
        let profile_id = validate_ref_id("profile", profile_id)?;
        let profile_path = self.paths.profile_document_path(&profile_id);
        canonicalize_file("profile document", &profile_path)
    }
}

pub(super) fn validate_ref_id(kind: &str, value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{kind} ref must not be empty"));
    }
    if trimmed != value {
        return Err(format!(
            "{kind} ref `{value}` must not contain leading or trailing whitespace"
        ));
    }
    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
    {
        return Err(format!(
            "{kind} ref `{value}` may only use ASCII letters, digits, `.`, `_`, and `-`"
        ));
    }

    Ok(trimmed.to_string())
}

pub(super) fn canonicalize_file(label: &str, path: &Path) -> Result<PathBuf, String> {
    if !path.is_absolute() {
        return Err(format!(
            "{label} `{}` must be an absolute path",
            path.display()
        ));
    }

    let canonical = fs::canonicalize(path)
        .map_err(|err| format!("failed to resolve {label} `{}`: {err}", path.display()))?;
    if !canonical.is_file() {
        return Err(format!("{label} `{}` is not a file", canonical.display()));
    }

    Ok(canonical)
}

pub(super) fn canonicalize_dir(label: &str, path: &Path) -> Result<PathBuf, String> {
    if !path.is_absolute() {
        return Err(format!(
            "{label} `{}` must be an absolute path",
            path.display()
        ));
    }

    let canonical = fs::canonicalize(path)
        .map_err(|err| format!("failed to resolve {label} `{}`: {err}", path.display()))?;
    if !canonical.is_dir() {
        return Err(format!(
            "{label} `{}` is not a directory",
            canonical.display()
        ));
    }

    Ok(canonical)
}
