use tauri::State;

use crate::{model::TickDetail, state::MoiraState};

#[tauri::command]
pub async fn receiver_status(state: State<'_, MoiraState>) -> Result<crate::model::ReceiverStatus, String> {
    state.receiver.snapshot(state.store.as_ref()).await
}

#[tauri::command]
pub async fn list_runs(state: State<'_, MoiraState>) -> Result<Vec<crate::model::RunSummary>, String> {
    state.store.list_runs().await
}

#[tauri::command]
pub async fn list_ticks(
    run_id: String,
    state: State<'_, MoiraState>,
) -> Result<Vec<crate::model::TickSummary>, String> {
    state.store.list_ticks(&run_id).await
}

#[tauri::command]
pub async fn tick_detail(
    run_id: String,
    tick: u64,
    state: State<'_, MoiraState>,
) -> Result<TickDetail, String> {
    state.store.tick_detail(&run_id, tick).await
}
