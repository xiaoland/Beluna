use tauri::State;

use crate::{app::state::AppState, atropos::model::RuntimeStatus, clotho::model::WakeInputRequest};

#[tauri::command]
pub async fn runtime_status(state: State<'_, AppState>) -> Result<RuntimeStatus, String> {
    state.atropos.runtime_status().await
}

#[tauri::command]
pub async fn wake(
    request: WakeInputRequest,
    state: State<'_, AppState>,
) -> Result<RuntimeStatus, String> {
    state.atropos.wake(request).await
}

#[tauri::command]
pub async fn stop(state: State<'_, AppState>) -> Result<RuntimeStatus, String> {
    state.atropos.stop().await
}

#[tauri::command]
pub async fn force_kill(state: State<'_, AppState>) -> Result<RuntimeStatus, String> {
    state.atropos.force_kill().await
}
