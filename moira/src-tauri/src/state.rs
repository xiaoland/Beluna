use std::sync::Arc;

use tokio::sync::RwLock;

use crate::{model::ReceiverStatus, store::MoiraStore};

#[derive(Debug, Clone)]
pub struct ReceiverSnapshot {
    pub wake_state: String,
    pub last_batch_at: Option<String>,
    pub last_error: Option<String>,
}

impl ReceiverSnapshot {
    fn new() -> Self {
        Self {
            wake_state: "awakening".to_string(),
            last_batch_at: None,
            last_error: None,
        }
    }
}

pub struct ReceiverState {
    endpoint: String,
    inner: RwLock<ReceiverSnapshot>,
}

impl ReceiverState {
    pub fn new(endpoint: String) -> Arc<Self> {
        Arc::new(Self {
            inner: RwLock::new(ReceiverSnapshot::new()),
            endpoint,
        })
    }

    pub async fn mark_faulted(&self, error: impl Into<String>) {
        let mut guard = self.inner.write().await;
        guard.wake_state = "faulted".to_string();
        guard.last_error = Some(error.into());
    }

    pub async fn mark_listening(&self) {
        let mut guard = self.inner.write().await;
        guard.wake_state = "listening".to_string();
        guard.last_error = None;
    }

    pub async fn mark_batch(&self, last_batch_at: String) {
        let mut guard = self.inner.write().await;
        guard.last_batch_at = Some(last_batch_at);
        if guard.wake_state == "awakening" || guard.wake_state == "listening" {
            guard.wake_state = "awake".to_string();
        }
    }

    pub async fn snapshot(&self, store: &MoiraStore) -> Result<ReceiverStatus, String> {
        let guard = self.inner.read().await.clone();
        let counts = store.counts().await?;
        Ok(ReceiverStatus {
            endpoint: self.endpoint.clone(),
            wake_state: guard.wake_state,
            db_path: store.db_path(),
            last_batch_at: guard.last_batch_at,
            last_error: guard.last_error,
            raw_event_count: counts.raw_event_count,
            wake_count: counts.run_count,
            tick_count: counts.tick_count,
        })
    }
}

pub struct MoiraState {
    pub store: Arc<MoiraStore>,
    pub receiver: Arc<ReceiverState>,
}

impl MoiraState {
    pub fn new(store: Arc<MoiraStore>, receiver: Arc<ReceiverState>) -> Self {
        Self { store, receiver }
    }
}
