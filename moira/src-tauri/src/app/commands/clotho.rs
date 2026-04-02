use tauri::State;

use crate::{
    app::state::AppState,
    clotho::model::{
        KnownLocalBuildRef, KnownLocalBuildRegistration, PreparedWakeInput, ProfileDocument,
        ProfileDocumentSummary, ProfileRef, SaveProfileDocumentRequest, WakeInputRequest,
    },
};

#[tauri::command]
pub async fn register_known_local_build(
    registration: KnownLocalBuildRegistration,
    state: State<'_, AppState>,
) -> Result<KnownLocalBuildRef, String> {
    state.clotho.register_known_local_build(registration)
}

#[tauri::command]
pub async fn prepare_wake_input(
    request: WakeInputRequest,
    state: State<'_, AppState>,
) -> Result<PreparedWakeInput, String> {
    state.clotho.prepare_wake_input(&request)
}

#[tauri::command]
pub async fn list_profile_documents(
    state: State<'_, AppState>,
) -> Result<Vec<ProfileDocumentSummary>, String> {
    state.clotho.list_profile_documents()
}

#[tauri::command]
pub async fn load_profile_document(
    profile: ProfileRef,
    state: State<'_, AppState>,
) -> Result<ProfileDocument, String> {
    state.clotho.load_profile_document(&profile)
}

#[tauri::command]
pub async fn save_profile_document(
    request: SaveProfileDocumentRequest,
    state: State<'_, AppState>,
) -> Result<ProfileDocument, String> {
    state.clotho.save_profile_document(request)
}
