use tauri::State;

use crate::{
    app::state::AppState,
    clotho::model::{
        ForgeLocalBuildRequest, InstallPublishedReleaseRequest, KnownLocalBuildRegistration,
        LaunchTargetRef, LaunchTargetSummary, PreparedWakeInput, ProfileDocument,
        ProfileDocumentSummary, ProfileRef, PublishedReleaseSummary, SaveProfileDocumentRequest,
        WakeInputRequest,
    },
};

#[tauri::command]
pub async fn register_known_local_build(
    registration: KnownLocalBuildRegistration,
    state: State<'_, AppState>,
) -> Result<LaunchTargetRef, String> {
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
pub async fn forge_local_build(
    request: ForgeLocalBuildRequest,
    state: State<'_, AppState>,
) -> Result<LaunchTargetRef, String> {
    state.clotho.forge_local_build(request).await
}

#[tauri::command]
pub async fn list_launch_targets(
    state: State<'_, AppState>,
) -> Result<Vec<LaunchTargetSummary>, String> {
    state.clotho.list_launch_targets()
}

#[tauri::command]
pub async fn list_published_releases(
    state: State<'_, AppState>,
) -> Result<Vec<PublishedReleaseSummary>, String> {
    state.clotho.list_published_releases().await
}

#[tauri::command]
pub async fn install_published_release(
    request: InstallPublishedReleaseRequest,
    state: State<'_, AppState>,
) -> Result<LaunchTargetRef, String> {
    state.clotho.install_published_release(request).await
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
