use tauri::State;

use moira_runtime::lachesis::model::{ReceiverStatus, RunSummary, TickDetail, TickSummary};

use crate::app::state::AppState;

#[tauri::command]
pub async fn receiver_status(state: State<'_, AppState>) -> Result<ReceiverStatus, String> {
    state.runtime.lachesis().receiver_status().await
}

#[tauri::command]
pub async fn list_runs(state: State<'_, AppState>) -> Result<Vec<RunSummary>, String> {
    state.runtime.lachesis().list_runs().await
}

#[tauri::command]
pub async fn list_ticks(
    run_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<TickSummary>, String> {
    state.runtime.lachesis().list_ticks(&run_id).await
}

#[tauri::command]
pub async fn tick_detail(
    run_id: String,
    tick: u64,
    state: State<'_, AppState>,
) -> Result<TickDetail, String> {
    state.runtime.lachesis().tick_detail(&run_id, tick).await
}
