use std::net::SocketAddr;
use std::sync::Arc;

use tauri::{AppHandle, Manager, RunEvent};

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
            crate::app::commands::clotho::register_known_local_build,
            crate::app::commands::clotho::prepare_wake_input,
            crate::app::commands::clotho::forge_local_build,
            crate::app::commands::clotho::list_launch_targets,
            crate::app::commands::clotho::list_published_releases,
            crate::app::commands::clotho::install_published_release,
            crate::app::commands::clotho::list_profile_documents,
            crate::app::commands::clotho::load_profile_document,
            crate::app::commands::clotho::save_profile_document,
            crate::app::commands::atropos::runtime_status,
            crate::app::commands::atropos::wake,
            crate::app::commands::atropos::stop,
            crate::app::commands::atropos::force_kill,
            crate::app::commands::lachesis::receiver_status,
            crate::app::commands::lachesis::list_runs,
            crate::app::commands::lachesis::list_ticks,
            crate::app::commands::lachesis::tick_detail
        ])
        .build(tauri::generate_context!())
        .expect("error while building Moira")
        .run(|app_handle, event| {
            if let RunEvent::ExitRequested { .. } = event {
                let managed = app_handle.state::<AppState>();
                if let Err(err) = tauri::async_runtime::block_on(managed.atropos.stop_if_running()) {
                    tracing::warn!(target: "moira::atropos", error = %err, "failed_to_request_stop_on_app_exit");
                }
            }
        });
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
    let atropos = Arc::new(AtroposService::new(paths, clotho.clone(), lachesis.clone()));

    Ok(AppState::new(clotho, lachesis, atropos))
}

fn parse_endpoint(endpoint: &str) -> Result<SocketAddr, String> {
    endpoint
        .parse::<SocketAddr>()
        .map_err(|err| format!("invalid OTLP endpoint `{endpoint}`: {err}"))
}
