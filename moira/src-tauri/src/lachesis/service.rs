use std::{net::SocketAddr, path::Path, sync::Arc};

use tauri::AppHandle;

use crate::lachesis::{
    model::{ReceiverStatus, RunSummary, TickDetail, TickSummary},
    receiver::{ReceiverState, start_otlp_logs_receiver},
    store::LachesisStore,
};

pub struct LachesisService {
    store: Arc<LachesisStore>,
    receiver: Arc<ReceiverState>,
}

impl LachesisService {
    pub async fn open(
        db_path: impl AsRef<Path>,
        receiver_bind: String,
    ) -> Result<Arc<Self>, String> {
        let store = LachesisStore::open(db_path.as_ref()).await?;
        let receiver = ReceiverState::new(receiver_bind);

        Ok(Arc::new(Self { store, receiver }))
    }

    pub async fn start_receiver(&self, endpoint: SocketAddr, app_handle: AppHandle) {
        start_otlp_logs_receiver(
            endpoint,
            self.receiver.clone(),
            self.store.clone(),
            app_handle,
        )
        .await;
    }

    pub async fn receiver_status(&self) -> Result<ReceiverStatus, String> {
        self.receiver.snapshot(self.store.as_ref()).await
    }

    pub async fn list_runs(&self) -> Result<Vec<RunSummary>, String> {
        self.store.list_runs().await
    }

    pub async fn list_ticks(&self, run_id: &str) -> Result<Vec<TickSummary>, String> {
        self.store.list_ticks(run_id).await
    }

    pub async fn tick_detail(&self, run_id: &str, tick: u64) -> Result<TickDetail, String> {
        self.store.tick_detail(run_id, tick).await
    }
}
