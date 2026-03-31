use std::net::SocketAddr;
use std::sync::Arc;

use tauri::{AppHandle, Manager};

use crate::{
    app::state::{AppPaths, AppState},
    atropos::AtroposService,
    clotho::ClothoService,
    lachesis::LachesisService,
};

const OTLP_ENDPOINT: &str = "127.0.0.1:4317";

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

            let managed = app.state::<AppState>();
            let lachesis = managed.lachesis.clone();
            let app_handle = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                lachesis
                    .start_receiver(
                        parse_endpoint(OTLP_ENDPOINT).expect("valid OTLP endpoint"),
                        app_handle,
                    )
                    .await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            crate::app::commands::lachesis::receiver_status,
            crate::app::commands::lachesis::list_runs,
            crate::app::commands::lachesis::list_ticks,
            crate::app::commands::lachesis::tick_detail
        ])
        .run(tauri::generate_context!())
        .expect("error while running Moira");
}

async fn initialize_state(app: &AppHandle) -> Result<AppState, String> {
    let app_root = app
        .path()
        .app_local_data_dir()
        .map_err(|err| format!("failed to resolve Moira app data directory: {err}"))?;
    let paths = AppPaths::from_root(app_root);
    paths.ensure_dirs()?;

    let clotho = Arc::new(ClothoService::new(paths.clone()));
    let lachesis =
        LachesisService::open(paths.telemetry_db_path(), OTLP_ENDPOINT.to_string()).await?;
    let atropos = Arc::new(AtroposService::new(paths));

    Ok(AppState::new(clotho, lachesis, atropos))
}

fn parse_endpoint(endpoint: &str) -> Result<SocketAddr, String> {
    endpoint
        .parse::<SocketAddr>()
        .map_err(|err| format!("invalid OTLP endpoint `{endpoint}`: {err}"))
}
