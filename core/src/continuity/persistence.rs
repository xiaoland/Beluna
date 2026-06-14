use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use opendal::{Error as OpenDalError, ErrorKind as OpenDalErrorKind, Operator, blocking, services};
use serde::{Deserialize, Serialize};
use tokio::runtime::{Builder as TokioRuntimeBuilder, Runtime};

use crate::continuity::{
    error::{ContinuityError, internal_error},
    types::{CONTINUITY_STORE_ENVELOPE_VERSION, ContinuityStore},
};

static OPENDAL_BLOCKING_RUNTIME: OnceLock<Runtime> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct ContinuityPersistence {
    path: PathBuf,
    object_key: String,
    operator: blocking::Operator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedContinuityState {
    version: u64,
    #[serde(default)]
    records: ContinuityStore,
}

#[derive(Debug)]
enum StorageCallError {
    OpenDal(OpenDalError),
    Panic,
}

impl ContinuityPersistence {
    pub fn new(path: PathBuf) -> Result<Self, ContinuityError> {
        let (root, object_key) = split_state_path(&path)?;
        let builder = services::Fs::default().root(&root);
        let operator = Operator::new(builder)
            .map_err(|err| map_opendal_error("create continuity fs operator", &path, err))?
            .finish();
        let operator = build_blocking_operator(operator, &path)?;
        Ok(Self {
            path,
            object_key,
            operator,
        })
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn load(&self) -> Result<Option<ContinuityStore>, ContinuityError> {
        let object_key = self.object_key.clone();
        let content = match self.run_storage_call(move |operator| {
            operator.read(&object_key).map(|content| content.to_vec())
        }) {
            Ok(content) => content,
            Err(StorageCallError::OpenDal(err)) if err.kind() == OpenDalErrorKind::NotFound => {
                return Ok(None);
            }
            Err(err) => return Err(self.map_storage_call_error("read continuity state", err)),
        };

        let parsed: PersistedContinuityState = serde_json::from_slice(&content).map_err(|err| {
            internal_error(format!(
                "failed to parse continuity state '{}': {err}",
                self.path.display()
            ))
        })?;
        if parsed.version != CONTINUITY_STORE_ENVELOPE_VERSION {
            return Err(internal_error(format!(
                "unsupported continuity state version {} at '{}'",
                parsed.version,
                self.path.display()
            )));
        }
        parsed.records.validate()?;

        Ok(Some(parsed.records))
    }

    pub fn save(&self, store: &ContinuityStore) -> Result<(), ContinuityError> {
        store.validate()?;
        let persisted = PersistedContinuityState {
            version: CONTINUITY_STORE_ENVELOPE_VERSION,
            records: store.clone(),
        };
        let mut content = serde_json::to_vec_pretty(&persisted).map_err(|err| {
            internal_error(format!(
                "failed to serialize continuity state '{}': {err}",
                self.path.display()
            ))
        })?;
        content.push(b'\n');

        let object_key = self.object_key.clone();
        self.run_storage_call(move |operator| operator.write(&object_key, content).map(|_| ()))
            .map_err(|err| self.map_storage_call_error("write continuity state", err))
    }

    fn run_storage_call<T, F>(&self, call: F) -> Result<T, StorageCallError>
    where
        T: Send + 'static,
        F: FnOnce(blocking::Operator) -> Result<T, OpenDalError> + Send + 'static,
    {
        let operator = self.operator.clone();
        if tokio::runtime::Handle::try_current().is_ok() {
            return std::thread::spawn(move || call(operator))
                .join()
                .map_err(|_| StorageCallError::Panic)?
                .map_err(StorageCallError::OpenDal);
        }

        call(operator).map_err(StorageCallError::OpenDal)
    }

    fn map_storage_call_error(
        &self,
        operation: &'static str,
        err: StorageCallError,
    ) -> ContinuityError {
        match err {
            StorageCallError::OpenDal(err) => {
                map_opendal_error(operation, self.path.as_path(), err)
            }
            StorageCallError::Panic => {
                internal_error(format!("{operation} '{}' panicked", self.path.display()))
            }
        }
    }
}

fn split_state_path(path: &Path) -> Result<(String, String), ContinuityError> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let root = parent.to_str().ok_or_else(|| {
        internal_error(format!(
            "continuity state root '{}' is not valid UTF-8",
            parent.display()
        ))
    })?;
    let object_key = path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .ok_or_else(|| {
            internal_error(format!(
                "continuity state path '{}' has no file name",
                path.display()
            ))
        })?;
    if object_key.trim().is_empty() {
        return Err(internal_error(format!(
            "continuity state path '{}' has empty file name",
            path.display()
        )));
    }

    Ok((root.to_string(), object_key.to_string()))
}

fn build_blocking_operator(
    operator: Operator,
    path: &Path,
) -> Result<blocking::Operator, ContinuityError> {
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        let _guard = handle.enter();
        return blocking::Operator::new(operator)
            .map_err(|err| map_opendal_error("create continuity blocking operator", path, err));
    }

    let runtime = opendal_blocking_runtime()?;
    let _guard = runtime.enter();
    blocking::Operator::new(operator)
        .map_err(|err| map_opendal_error("create continuity blocking operator", path, err))
}

fn opendal_blocking_runtime() -> Result<&'static Runtime, ContinuityError> {
    if let Some(runtime) = OPENDAL_BLOCKING_RUNTIME.get() {
        return Ok(runtime);
    }

    let runtime = TokioRuntimeBuilder::new_multi_thread()
        .enable_all()
        .thread_name("continuity-opendal-blocking")
        .build()
        .map_err(|err| {
            internal_error(format!(
                "failed to build continuity OpenDAL blocking runtime: {err}"
            ))
        })?;
    let _ = OPENDAL_BLOCKING_RUNTIME.set(runtime);
    Ok(OPENDAL_BLOCKING_RUNTIME
        .get()
        .expect("OpenDAL blocking runtime should be initialized"))
}

fn map_opendal_error(operation: &'static str, path: &Path, err: OpenDalError) -> ContinuityError {
    internal_error(format!("{operation} '{}' failed: {err}", path.display()))
}
