use tauri::{AppHandle, Emitter};

use crate::lachesis::model::IngestPulse;

pub const LACHESIS_UPDATED_EVENT: &str = "lachesis-updated";

pub fn emit_lachesis_updated(app_handle: &AppHandle, pulse: IngestPulse) {
    let _ = app_handle.emit(LACHESIS_UPDATED_EVENT, pulse);
}

pub fn empty_ingest_pulse() -> IngestPulse {
    IngestPulse {
        touched_run_ids: Vec::new(),
        last_batch_at: String::new(),
    }
}
