use std::sync::Arc;

use moira_runtime::MoiraRuntime;

pub struct AppState {
    pub runtime: Arc<MoiraRuntime>,
}

impl AppState {
    pub fn new(runtime: Arc<MoiraRuntime>) -> Self {
        Self { runtime }
    }
}
