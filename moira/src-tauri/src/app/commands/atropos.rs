use tauri::State;

use moira_runtime::{atropos::model::RuntimeStatus, clotho::model::WakeInputRequest};

use crate::app::state::AppState;

#[tauri::command]
pub async fn runtime_status(state: State<'_, AppState>) -> Result<RuntimeStatus, String> {
    state.runtime.atropos().runtime_status().await
}

#[tauri::command]
pub async fn wake(
    request: WakeInputRequest,
    state: State<'_, AppState>,
) -> Result<RuntimeStatus, String> {
    state.runtime.atropos().wake(request).await
}

#[tauri::command]
pub async fn stop(state: State<'_, AppState>) -> Result<RuntimeStatus, String> {
    state.runtime.atropos().stop().await
}

#[tauri::command]
pub async fn force_kill(state: State<'_, AppState>) -> Result<RuntimeStatus, String> {
    state.runtime.atropos().force_kill().await
}
