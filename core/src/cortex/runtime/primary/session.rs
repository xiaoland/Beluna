use std::sync::Arc;

use tokio::sync::Mutex;

use crate::ai_gateway::chat::Thread;

use super::{PrimaryContinuationState, PrimaryThreadState};

#[derive(Clone, Default)]
pub(super) struct PrimarySession {
    thread_state: Arc<Mutex<Option<PrimaryThreadState>>>,
    continuation_state: Arc<Mutex<Option<PrimaryContinuationState>>>,
}

impl PrimarySession {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) async fn thread(&self) -> Option<Thread> {
        self.thread_state
            .lock()
            .await
            .as_ref()
            .map(|state| state.thread.clone())
    }

    pub(super) async fn set_thread(&self, thread: Thread) {
        let mut guard = self.thread_state.lock().await;
        *guard = Some(PrimaryThreadState { thread });
    }

    pub(super) async fn take_continuation(&self) -> Option<PrimaryContinuationState> {
        self.continuation_state.lock().await.take()
    }

    pub(super) async fn set_continuation(&self, state: PrimaryContinuationState) {
        let mut guard = self.continuation_state.lock().await;
        *guard = Some(state);
    }

    pub(super) async fn clear_continuation(&self) {
        let mut guard = self.continuation_state.lock().await;
        *guard = None;
    }

    pub(super) async fn reset(&self) {
        self.clear_continuation().await;
        let mut guard = self.thread_state.lock().await;
        *guard = None;
    }
}
