use tauri::State;

use crate::{
    app::state::AppState,
    lachesis::model::{ReceiverStatus, RunSummary, TickDetail, TickSummary},
};

#[tauri::command]
pub async fn receiver_status(state: State<'_, AppState>) -> Result<ReceiverStatus, String> {
    state.lachesis.receiver_status().await
}

#[tauri::command]
pub async fn list_runs(state: State<'_, AppState>) -> Result<Vec<RunSummary>, String> {
    state.lachesis.list_runs().await
}

#[tauri::command]
pub async fn list_ticks(
    run_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<TickSummary>, String> {
    state.lachesis.list_ticks(&run_id).await
}

#[tauri::command]
pub async fn tick_detail(
    run_id: String,
    tick: u64,
    state: State<'_, AppState>,
) -> Result<TickDetail, String> {
    state.lachesis.tick_detail(&run_id, tick).await
}
