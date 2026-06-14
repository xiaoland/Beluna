use crate::{
    continuity::{
        error::ContinuityError,
        types::{ContinuityRecord, ContinuityRecordBody, ContinuityRecordKey, ContinuityStore},
    },
    types::{Act, DispatchDecision},
};

#[derive(Debug, Clone)]
pub struct ContinuityState {
    pub store: ContinuityStore,
}

impl Default for ContinuityState {
    fn default() -> Self {
        Self {
            store: ContinuityStore::default(),
        }
    }
}

impl ContinuityState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_store(store: ContinuityStore) -> Self {
        Self { store }
    }

    pub fn store_snapshot(&self) -> ContinuityStore {
        self.store.clone()
    }

    pub fn get_record(
        &self,
        key: &ContinuityRecordKey,
    ) -> Result<Option<ContinuityRecord>, ContinuityError> {
        self.store.get_record(key)
    }

    pub fn put_record(
        &mut self,
        key: ContinuityRecordKey,
        expected_revision: Option<u64>,
        schema_version: impl Into<String>,
        body: ContinuityRecordBody,
    ) -> Result<ContinuityRecord, ContinuityError> {
        self.store
            .put_record(key, expected_revision, schema_version, body)
    }

    pub fn delete_record(
        &mut self,
        key: &ContinuityRecordKey,
        expected_revision: Option<u64>,
    ) -> Result<Option<ContinuityRecord>, ContinuityError> {
        self.store.delete_record(key, expected_revision)
    }

    pub fn on_act(&self, _act: &Act) -> DispatchDecision {
        DispatchDecision::Continue
    }
}
