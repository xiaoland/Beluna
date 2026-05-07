use std::net::SocketAddr;
use std::sync::Arc;

use moira_runtime::{
    MoiraEvent, MoiraEventSink, MoiraPaths, MoiraRuntime, MoiraRuntimeConfig, MoiraTask,
    MoiraTaskSpawner,
};
use tauri::{AppHandle, Emitter, Manager, RunEvent};

use crate::app::state::AppState;

const OTLP_ENDPOINT: &str = "127.0.0.1:4317";
const LACHESIS_UPDATED_EVENT: &str = "lachesis-updated";

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
            let runtime = managed.runtime.clone();
            tauri::async_runtime::block_on(runtime.status()).map_err(
                |error| -> Box<dyn std::error::Error> { error.to_string().into() },
            )?;

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
                if let Err(err) = tauri::async_runtime::block_on(managed.runtime.shutdown()) {
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
    let runtime = MoiraRuntime::open(MoiraRuntimeConfig {
        paths: MoiraPaths::from_root(app_root),
        receiver_bind: parse_endpoint(OTLP_ENDPOINT)?,
        event_sink: Arc::new(TauriEventSink {
            app_handle: app.clone(),
        }),
        task_spawner: Arc::new(TauriTaskSpawner),
    })
    .await
    .map_err(|error| error.to_string())?;

    Ok(AppState::new(runtime))
}

fn parse_endpoint(endpoint: &str) -> Result<SocketAddr, String> {
    endpoint
        .parse::<SocketAddr>()
        .map_err(|err| format!("invalid OTLP endpoint `{endpoint}`: {err}"))
}

struct TauriEventSink {
    app_handle: AppHandle,
}

impl MoiraEventSink for TauriEventSink {
    fn emit(&self, event: MoiraEvent) {
        match event {
            MoiraEvent::LachesisUpdated(pulse) => {
                let _ = self.app_handle.emit(LACHESIS_UPDATED_EVENT, pulse);
            }
            MoiraEvent::ResourceStatusChanged(_) | MoiraEvent::CoreSupervisionChanged(_) => {}
        }
    }
}

struct TauriTaskSpawner;

impl MoiraTaskSpawner for TauriTaskSpawner {
    fn spawn(&self, task: MoiraTask) {
        tauri::async_runtime::spawn(task);
    }
}
