mod otlp;

use std::{net::SocketAddr, sync::Arc};

use tauri::{AppHandle, Emitter};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;

use crate::{
    model::IngestPulse,
    state::ReceiverState,
    store::MoiraStore,
};

pub async fn start_otlp_logs_receiver(
    endpoint: SocketAddr,
    receiver: Arc<ReceiverState>,
    store: Arc<MoiraStore>,
    app_handle: AppHandle,
) {
    let service = otlp::MoiraLogsService::new(receiver.clone(), store, app_handle.clone());
    let listener = match TcpListener::bind(endpoint).await {
        Ok(listener) => {
            receiver.mark_listening().await;
            listener
        }
        Err(err) => {
            let message = format!("OTLP logs receiver failed to bind: {err}");
            receiver.mark_faulted(message.clone()).await;
            let _ = app_handle.emit(
                "lachesis-updated",
                IngestPulse {
                    touched_run_ids: Vec::new(),
                    last_batch_at: String::new(),
                },
            );
            return;
        }
    };

    match Server::builder()
        .add_service(otlp::logs_service(service))
        .serve_with_incoming(TcpListenerStream::new(listener))
        .await
    {
        Ok(()) => {}
        Err(err) => {
            let message = format!("OTLP logs receiver faulted: {err}");
            receiver.mark_faulted(message.clone()).await;
            let _ = app_handle.emit(
                "lachesis-updated",
                IngestPulse {
                    touched_run_ids: Vec::new(),
                    last_batch_at: String::new(),
                },
            );
        }
    }
}
