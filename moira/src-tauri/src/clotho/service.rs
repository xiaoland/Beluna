use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::app::state::AppPaths;

use super::model::{
    KnownLocalBuildRecord, KnownLocalBuildRef, KnownLocalBuildRegistration,
    PreparedKnownLocalBuild, PreparedWakeInput, WakeInputRequest,
};

#[derive(Debug)]
pub struct ClothoService {
    #[allow(dead_code)]
    paths: AppPaths,
}

impl ClothoService {
    pub fn new(paths: AppPaths) -> Self {
        Self { paths }
    }

    #[allow(dead_code)]
    pub fn paths(&self) -> &AppPaths {
        &self.paths
    }

    #[allow(dead_code)]
    pub fn register_known_local_build(
        &self,
        registration: KnownLocalBuildRegistration,
    ) -> Result<KnownLocalBuildRef, String> {
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

        let record = KnownLocalBuildRecord {
            build_id: build_id.clone(),
            executable_path,
            working_dir,
            source_dir,
        };
        let manifest_path = self.known_local_build_manifest_path(&build_id);

        fs::create_dir_all(self.paths.known_local_builds_dir()).map_err(|err| {
            format!(
                "failed to create known local builds directory `{}`: {err}",
                self.paths.known_local_builds_dir().display()
            )
        })?;

        let rendered = serde_json::to_string_pretty(&record)
            .map_err(|err| format!("failed to render build manifest: {err}"))?;
        fs::write(&manifest_path, format!("{rendered}\n")).map_err(|err| {
            format!(
                "failed to write known local build manifest `{}`: {err}",
                manifest_path.display()
            )
        })?;

        Ok(KnownLocalBuildRef { build_id })
    }

    #[allow(dead_code)]
    pub fn prepare_wake_input(
        &self,
        request: &WakeInputRequest,
    ) -> Result<PreparedWakeInput, String> {
        let build = self.resolve_known_local_build(&request.build)?;
        let profile_path = request
            .profile
            .as_ref()
            .map(|profile| self.resolve_profile_path(&profile.profile_id))
            .transpose()?;

        Ok(PreparedWakeInput {
            build,
            profile_path,
        })
    }

    #[allow(dead_code)]
    pub fn resolve_known_local_build(
        &self,
        build_ref: &KnownLocalBuildRef,
    ) -> Result<PreparedKnownLocalBuild, String> {
        let build_id = validate_ref_id("build", &build_ref.build_id)?;
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

        Ok(PreparedKnownLocalBuild {
            build_id,
            executable_path: canonicalize_file("build executable", &record.executable_path)?,
            working_dir: canonicalize_dir("build working directory", &record.working_dir)?,
            source_dir: record
                .source_dir
                .as_ref()
                .map(|path| canonicalize_dir("build source directory", path))
                .transpose()?,
        })
    }

    #[allow(dead_code)]
    fn resolve_profile_path(&self, profile_id: &str) -> Result<PathBuf, String> {
        let profile_id = validate_ref_id("profile", profile_id)?;
        let profile_path = self.paths.profile_document_path(&profile_id);
        canonicalize_file("profile document", &profile_path)
    }

    #[allow(dead_code)]
    fn known_local_build_manifest_path(&self, build_id: &str) -> PathBuf {
        self.paths
            .known_local_builds_dir()
            .join(format!("{build_id}.json"))
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

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use uuid::Uuid;

    use crate::app::state::AppPaths;

    use super::{
        super::model::{KnownLocalBuildRegistration, WakeInputRequest},
        ClothoService,
    };

    #[test]
    fn register_and_prepare_known_local_build_round_trip() {
        let sandbox = TestSandbox::new();
        let paths = AppPaths::from_root(sandbox.root.clone());
        paths.ensure_dirs().expect("app paths should initialize");

        let executable_path = sandbox.create_file("fixtures/builds/dev-core/bin/beluna-core");
        let source_dir = sandbox.create_dir("fixtures/builds/dev-core/src");
        let profile_path = sandbox.create_file("profiles/default.jsonc");
        assert!(profile_path.exists());

        let service = ClothoService::new(paths);
        let build_ref = service
            .register_known_local_build(KnownLocalBuildRegistration {
                build_id: "dev-core".to_string(),
                executable_path: executable_path.clone(),
                working_dir: None,
                source_dir: Some(source_dir.clone()),
            })
            .expect("known local build should register");
        let prepared = service
            .prepare_wake_input(&WakeInputRequest {
                build: build_ref.clone(),
                profile: Some(super::super::model::ProfileRef {
                    profile_id: "default".to_string(),
                }),
            })
            .expect("wake input should resolve");

        assert_eq!(prepared.build.build_id, "dev-core");
        assert_eq!(
            prepared.build.executable_path,
            fs::canonicalize(executable_path).expect("executable path should resolve")
        );
        assert_eq!(
            prepared.build.source_dir,
            Some(fs::canonicalize(source_dir).expect("source dir should resolve"))
        );
        assert_eq!(
            prepared.profile_path,
            Some(fs::canonicalize(profile_path).expect("profile path should resolve"))
        );

        let manifest_path = service
            .paths()
            .known_local_builds_dir()
            .join(format!("{}.json", build_ref.build_id));
        assert!(manifest_path.is_file());
    }

    #[test]
    fn register_rejects_relative_executable_paths() {
        let sandbox = TestSandbox::new();
        let paths = AppPaths::from_root(sandbox.root.clone());
        paths.ensure_dirs().expect("app paths should initialize");
        let service = ClothoService::new(paths);

        let err = service
            .register_known_local_build(KnownLocalBuildRegistration {
                build_id: "dev-core".to_string(),
                executable_path: PathBuf::from("relative/beluna-core"),
                working_dir: None,
                source_dir: None,
            })
            .expect_err("relative executable path should fail");

        assert!(err.contains("absolute path"));
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
            let path = self.root.join(relative);
            let parent = path.parent().expect("file should have parent");
            fs::create_dir_all(parent).expect("file parent should create");
            fs::write(&path, "#!/bin/sh\nexit 0\n").expect("file should write");
            path
        }
    }

    impl Drop for TestSandbox {
        fn drop(&mut self) {
            let _ = remove_dir_all_if_exists(&self.root);
        }
    }

    fn remove_dir_all_if_exists(path: &Path) -> std::io::Result<()> {
        if path.exists() {
            fs::remove_dir_all(path)?;
        }
        Ok(())
    }
}
