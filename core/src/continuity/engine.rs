use std::path::PathBuf;

use crate::{
    continuity::{
        error::ContinuityError,
        persistence::ContinuityPersistence,
        state::ContinuityState,
        types::{ContinuityRecord, ContinuityRecordBody, ContinuityRecordKey, DispatchContext},
    },
    types::{Act, DispatchDecision},
};

#[derive(Clone)]
pub struct ContinuityEngine {
    state: ContinuityState,
    persistence: ContinuityPersistence,
}

impl ContinuityEngine {
    pub fn new(state: ContinuityState, persistence: ContinuityPersistence) -> Self {
        Self { state, persistence }
    }

    pub fn with_defaults_at(path: PathBuf) -> Result<Self, ContinuityError> {
        let persistence = ContinuityPersistence::new(path)?;
        let store = persistence.load()?.unwrap_or_default();
        store.validate()?;

        Ok(Self {
            state: ContinuityState::with_store(store),
            persistence,
        })
    }

    pub fn state(&self) -> &ContinuityState {
        &self.state
    }

    pub fn get_record(
        &self,
        key: &ContinuityRecordKey,
    ) -> Result<Option<ContinuityRecord>, ContinuityError> {
        self.state.get_record(key)
    }

    pub fn put_record(
        &mut self,
        key: ContinuityRecordKey,
        expected_revision: Option<u64>,
        schema_version: impl Into<String>,
        body: ContinuityRecordBody,
    ) -> Result<ContinuityRecord, ContinuityError> {
        let record = self
            .state
            .put_record(key, expected_revision, schema_version, body)?;
        self.flush()?;
        Ok(record)
    }

    pub fn delete_record(
        &mut self,
        key: &ContinuityRecordKey,
        expected_revision: Option<u64>,
    ) -> Result<Option<ContinuityRecord>, ContinuityError> {
        let deleted = self.state.delete_record(key, expected_revision)?;
        self.flush()?;
        Ok(deleted)
    }

    pub fn on_act(
        &self,
        act: &Act,
        _ctx: &DispatchContext,
    ) -> Result<DispatchDecision, ContinuityError> {
        Ok(self.state.on_act(act))
    }

    pub fn flush(&mut self) -> Result<(), ContinuityError> {
        self.persistence.save(&self.state.store)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use serde_json::Value;
    use uuid::Uuid;

    use super::*;
    use crate::continuity::types::{
        CONTINUITY_STORE_ENVELOPE_VERSION, ContinuityRecordBody, ContinuityRecordKey,
    };

    #[test]
    fn engine_persists_and_reloads_generic_records() -> Result<(), Box<dyn std::error::Error>> {
        let path = unique_state_path("generic-record");
        let key = ContinuityRecordKey::new("test.namespace", "binary");

        let mut engine = ContinuityEngine::with_defaults_at(path.clone())?;
        engine.put_record(
            key.clone(),
            None,
            "binary.v1",
            ContinuityRecordBody::new("application/octet-stream", vec![0, 1, 255]),
        )?;

        let reloaded = ContinuityEngine::with_defaults_at(path.clone())?;
        let record = reloaded
            .get_record(&key)?
            .expect("persisted record should reload");
        assert_eq!(record.revision, 1);
        assert_eq!(record.schema_version, "binary.v1");
        assert_eq!(record.body.content_type, "application/octet-stream");
        assert_eq!(record.body.bytes, vec![0, 1, 255]);

        let raw = fs::read_to_string(&path)?;
        let value: Value = serde_json::from_str(&raw)?;
        assert_eq!(value["version"], CONTINUITY_STORE_ENVELOPE_VERSION);
        assert_eq!(
            value["records"]["test.namespace"]["binary"]["body"]["bytes"],
            serde_json::json!([0, 1, 255])
        );

        let _ = fs::remove_file(&path);
        if let Some(parent) = path.parent() {
            let _ = fs::remove_dir(parent);
        }
        Ok(())
    }

    fn unique_state_path(label: &str) -> PathBuf {
        std::env::temp_dir()
            .join(format!("beluna-continuity-{label}-{}", Uuid::new_v4()))
            .join("state.json")
    }
}
