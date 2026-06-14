use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::continuity::error::{
    ContinuityError, arithmetic_error, invalid_request, ledger_conflict,
};

pub const CONTINUITY_STORE_ENVELOPE_VERSION: u64 = 1;
pub const INITIAL_RECORD_REVISION: u64 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalDebitObservation {
    pub reference_id: String,
    pub cost_attribution_id: String,
    #[serde(default)]
    pub action_id: Option<String>,
    #[serde(default)]
    pub cycle_id: Option<u64>,
    pub debit_survival_micro: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchContext {
    pub cycle_id: u64,
    pub act_seq_no: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ContinuityRecordKey {
    pub namespace: String,
    pub record_id: String,
}

impl ContinuityRecordKey {
    pub fn new(namespace: impl Into<String>, record_id: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            record_id: record_id.into(),
        }
    }

    pub fn validate(&self) -> Result<(), ContinuityError> {
        validate_key_part("namespace", &self.namespace)?;
        validate_key_part("record id", &self.record_id)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContinuityRecordBody {
    pub content_type: String,
    pub bytes: Vec<u8>,
}

impl ContinuityRecordBody {
    pub fn new(content_type: impl Into<String>, bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            content_type: content_type.into(),
            bytes: bytes.into(),
        }
    }

    fn validate(&self) -> Result<(), ContinuityError> {
        if self.content_type.trim().is_empty() {
            return Err(invalid_request("record content_type cannot be empty"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContinuityRecord {
    pub revision: u64,
    pub schema_version: String,
    pub body: ContinuityRecordBody,
}

impl ContinuityRecord {
    fn validate(&self) -> Result<(), ContinuityError> {
        if self.revision < INITIAL_RECORD_REVISION {
            return Err(invalid_request(format!(
                "record revision must be at least {INITIAL_RECORD_REVISION}"
            )));
        }
        if self.schema_version.trim().is_empty() {
            return Err(invalid_request("record schema_version cannot be empty"));
        }
        self.body.validate()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContinuityStore {
    pub namespaces: BTreeMap<String, BTreeMap<String, ContinuityRecord>>,
}

impl ContinuityStore {
    pub fn get_record(
        &self,
        key: &ContinuityRecordKey,
    ) -> Result<Option<ContinuityRecord>, ContinuityError> {
        key.validate()?;
        Ok(self
            .namespaces
            .get(&key.namespace)
            .and_then(|records| records.get(&key.record_id))
            .cloned())
    }

    pub fn put_record(
        &mut self,
        key: ContinuityRecordKey,
        expected_revision: Option<u64>,
        schema_version: impl Into<String>,
        body: ContinuityRecordBody,
    ) -> Result<ContinuityRecord, ContinuityError> {
        key.validate()?;
        body.validate()?;

        let schema_version = schema_version.into();
        if schema_version.trim().is_empty() {
            return Err(invalid_request("record schema_version cannot be empty"));
        }

        let namespace = self.namespaces.entry(key.namespace.clone()).or_default();
        let current_revision = namespace.get(&key.record_id).map(|record| record.revision);
        validate_expected_revision(&key, expected_revision, current_revision)?;

        let revision = match current_revision {
            Some(revision) => revision
                .checked_add(1)
                .ok_or_else(|| arithmetic_error("record revision overflow"))?,
            None => INITIAL_RECORD_REVISION,
        };
        let record = ContinuityRecord {
            revision,
            schema_version,
            body,
        };
        record.validate()?;
        namespace.insert(key.record_id, record.clone());
        Ok(record)
    }

    pub fn delete_record(
        &mut self,
        key: &ContinuityRecordKey,
        expected_revision: Option<u64>,
    ) -> Result<Option<ContinuityRecord>, ContinuityError> {
        key.validate()?;
        let current_revision = self
            .namespaces
            .get(&key.namespace)
            .and_then(|records| records.get(&key.record_id))
            .map(|record| record.revision);
        validate_expected_revision(key, expected_revision, current_revision)?;

        let Some(namespace) = self.namespaces.get_mut(&key.namespace) else {
            return Ok(None);
        };
        let removed = namespace.remove(&key.record_id);
        if namespace.is_empty() {
            self.namespaces.remove(&key.namespace);
        }
        Ok(removed)
    }

    pub fn validate(&self) -> Result<(), ContinuityError> {
        for (namespace, records) in &self.namespaces {
            validate_key_part("namespace", namespace)?;
            for (record_id, record) in records {
                validate_key_part("record id", record_id)?;
                record.validate()?;
            }
        }
        Ok(())
    }
}

fn validate_key_part(label: &str, value: &str) -> Result<(), ContinuityError> {
    if value.trim().is_empty() {
        return Err(invalid_request(format!("record {label} cannot be empty")));
    }
    if value.contains('/') {
        return Err(invalid_request(format!(
            "record {label} cannot contain '/'"
        )));
    }
    Ok(())
}

fn validate_expected_revision(
    key: &ContinuityRecordKey,
    expected_revision: Option<u64>,
    current_revision: Option<u64>,
) -> Result<(), ContinuityError> {
    let Some(expected_revision) = expected_revision else {
        return Ok(());
    };
    if current_revision == Some(expected_revision) {
        return Ok(());
    }
    Err(ledger_conflict(format!(
        "record revision conflict for {}/{}: expected {}, got {}",
        key.namespace,
        key.record_id,
        expected_revision,
        current_revision
            .map(|revision| revision.to_string())
            .unwrap_or_else(|| "missing".to_string())
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::continuity::ContinuityErrorKind;

    #[test]
    fn store_put_get_delete_tracks_revisions_and_bytes() {
        let mut store = ContinuityStore::default();
        let key = ContinuityRecordKey::new("test.namespace", "binary");

        let created = store
            .put_record(
                key.clone(),
                None,
                "binary.v1",
                ContinuityRecordBody::new("application/octet-stream", vec![0, 1, 255]),
            )
            .expect("create record");
        assert_eq!(created.revision, 1);
        assert_eq!(created.body.bytes, vec![0, 1, 255]);

        let replaced = store
            .put_record(
                key.clone(),
                Some(1),
                "binary.v1",
                ContinuityRecordBody::new("application/octet-stream", vec![2, 3, 4]),
            )
            .expect("replace record");
        assert_eq!(replaced.revision, 2);
        assert_eq!(store.get_record(&key).expect("get record"), Some(replaced));

        let conflict = store
            .put_record(
                key.clone(),
                Some(1),
                "binary.v1",
                ContinuityRecordBody::new("application/octet-stream", vec![5]),
            )
            .expect_err("stale revision should fail");
        assert_eq!(conflict.kind, ContinuityErrorKind::LedgerConflict);

        let deleted = store
            .delete_record(&key, Some(2))
            .expect("delete record")
            .expect("deleted record");
        assert_eq!(deleted.revision, 2);
        assert_eq!(store.get_record(&key).expect("get missing"), None);
    }
}
