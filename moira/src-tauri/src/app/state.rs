use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::{atropos::AtroposService, clotho::ClothoService, lachesis::LachesisService};

#[derive(Debug, Clone)]
pub struct AppPaths {
    root: PathBuf,
}

impl AppPaths {
    pub fn from_root(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn artifacts_dir(&self) -> PathBuf {
        self.root.join("artifacts")
    }

    pub fn known_local_builds_dir(&self) -> PathBuf {
        self.artifacts_dir().join("known-local-builds")
    }

    pub fn profiles_dir(&self) -> PathBuf {
        self.root.join("profiles")
    }

    #[allow(dead_code)]
    pub fn profile_document_path(&self, profile_id: &str) -> PathBuf {
        self.profiles_dir().join(format!("{profile_id}.jsonc"))
    }

    pub fn runtime_dir(&self) -> PathBuf {
        self.root.join("runtime")
    }

    pub fn cache_dir(&self) -> PathBuf {
        self.root.join("cache")
    }

    pub fn telemetry_dir(&self) -> PathBuf {
        self.root.join("telemetry")
    }

    pub fn telemetry_db_path(&self) -> PathBuf {
        self.telemetry_dir().join("moira.duckdb")
    }

    pub fn ensure_dirs(&self) -> Result<(), String> {
        for path in [
            self.root().to_path_buf(),
            self.artifacts_dir(),
            self.known_local_builds_dir(),
            self.profiles_dir(),
            self.runtime_dir(),
            self.cache_dir(),
            self.telemetry_dir(),
        ] {
            std::fs::create_dir_all(&path).map_err(|err| {
                format!(
                    "failed to create Moira app directory `{}`: {err}",
                    path.display()
                )
            })?;
        }

        Ok(())
    }
}

pub struct AppState {
    #[allow(dead_code)]
    pub clotho: Arc<ClothoService>,
    pub lachesis: Arc<LachesisService>,
    #[allow(dead_code)]
    pub atropos: Arc<AtroposService>,
}

impl AppState {
    pub fn new(
        clotho: Arc<ClothoService>,
        lachesis: Arc<LachesisService>,
        atropos: Arc<AtroposService>,
    ) -> Self {
        Self {
            clotho,
            lachesis,
            atropos,
        }
    }
}
