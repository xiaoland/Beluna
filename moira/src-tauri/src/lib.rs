mod commands;
mod ingest;
mod model;
mod state;
mod store;

use std::net::SocketAddr;

use state::{MoiraState, ReceiverState};
use store::MoiraStore;
use tauri::{AppHandle, Manager};

const OTLP_ENDPOINT: &str = "127.0.0.1:4317";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                let state = initialize_state(&app_handle).await?;
                app_handle.manage(state);
                Ok::<(), String>(())
            })
            .map_err(|error| -> Box<dyn std::error::Error> { error.into() })?;

            let managed = app.state::<MoiraState>();
            let receiver = managed.receiver.clone();
            let store = managed.store.clone();
            let app_handle = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                ingest::start_otlp_logs_receiver(
                    parse_endpoint(OTLP_ENDPOINT).expect("valid OTLP endpoint"),
                    receiver,
                    store,
                    app_handle,
                )
                .await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::receiver_status,
            commands::list_runs,
            commands::list_ticks,
            commands::tick_detail
        ])
        .run(tauri::generate_context!())
        .expect("error while running Moira");
}

async fn initialize_state(app: &AppHandle) -> Result<MoiraState, String> {
    let app_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|err| format!("failed to resolve Moira app data directory: {err}"))?;
    std::fs::create_dir_all(&app_dir)
        .map_err(|err| format!("failed to create Moira app data directory: {err}"))?;

    let db_path = app_dir.join("telemetry").join("moira.duckdb");
    let store = MoiraStore::open(&db_path).await?;
    let receiver = ReceiverState::new(OTLP_ENDPOINT.to_string());

    Ok(MoiraState::new(store, receiver))
}

fn parse_endpoint(endpoint: &str) -> Result<SocketAddr, String> {
    endpoint
        .parse::<SocketAddr>()
        .map_err(|err| format!("invalid OTLP endpoint `{endpoint}`: {err}"))
}
