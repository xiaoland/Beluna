use std::{
    fs,
    io::{BufWriter, Write},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::{
    continuity::error::{ContinuityError, internal_error},
    cortex::CognitionState,
};

const PERSISTENCE_VERSION: u64 = 1;

#[derive(Debug, Clone)]
pub struct ContinuityPersistence {
    path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedContinuityState {
    version: u64,
    cognition_state: CognitionState,
}

impl ContinuityPersistence {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn load(&self) -> Result<Option<CognitionState>, ContinuityError> {
        let content = match fs::read_to_string(&self.path) {
            Ok(content) => content,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(err) => {
                return Err(internal_error(format!(
                    "failed to read continuity state '{}': {err}",
                    self.path.display()
                )));
            }
        };

        let parsed: PersistedContinuityState = serde_json::from_str(&content).map_err(|err| {
            internal_error(format!(
                "failed to parse continuity state '{}': {err}",
                self.path.display()
            ))
        })?;
        if parsed.version != PERSISTENCE_VERSION {
            return Err(internal_error(format!(
                "unsupported continuity state version {} at '{}'",
                parsed.version,
                self.path.display()
            )));
        }

        Ok(Some(parsed.cognition_state))
    }

    pub fn save(&self, cognition_state: &CognitionState) -> Result<(), ContinuityError> {
        let parent = self.path.parent().ok_or_else(|| {
            internal_error(format!(
                "continuity state path '{}' has no parent",
                self.path.display()
            ))
        })?;
        fs::create_dir_all(parent).map_err(|err| {
            internal_error(format!(
                "failed to create continuity state directory '{}': {err}",
                parent.display()
            ))
        })?;

        let persisted = PersistedContinuityState {
            version: PERSISTENCE_VERSION,
            cognition_state: cognition_state.clone(),
        };

        let tmp_path = self.path.with_extension("tmp");
        let file = fs::File::create(&tmp_path).map_err(|err| {
            internal_error(format!(
                "failed to create continuity temp file '{}': {err}",
                tmp_path.display()
            ))
        })?;
        {
            let mut writer = BufWriter::new(file);
            serde_json::to_writer_pretty(&mut writer, &persisted).map_err(|err| {
                internal_error(format!(
                    "failed to serialize continuity state '{}': {err}",
                    tmp_path.display()
                ))
            })?;
            writer.write_all(b"\n").map_err(|err| {
                internal_error(format!(
                    "failed to finalize continuity state '{}': {err}",
                    tmp_path.display()
                ))
            })?;
            writer.flush().map_err(|err| {
                internal_error(format!(
                    "failed to flush continuity state '{}': {err}",
                    tmp_path.display()
                ))
            })?;
        }

        let tmp_file = fs::OpenOptions::new()
            .read(true)
            .open(&tmp_path)
            .map_err(|err| {
                internal_error(format!(
                    "failed to reopen continuity temp file '{}': {err}",
                    tmp_path.display()
                ))
            })?;
        tmp_file.sync_all().map_err(|err| {
            internal_error(format!(
                "failed to sync continuity temp file '{}': {err}",
                tmp_path.display()
            ))
        })?;

        fs::rename(&tmp_path, &self.path).map_err(|err| {
            internal_error(format!(
                "failed to replace continuity state '{}' from '{}': {err}",
                self.path.display(),
                tmp_path.display()
            ))
        })?;

        if let Ok(parent_file) = fs::File::open(parent) {
            let _ = parent_file.sync_all();
        }

        Ok(())
    }
}
