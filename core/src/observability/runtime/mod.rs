mod cortex;
mod emit;
mod flatten;
#[cfg(test)]
mod flatten_tests;
mod spine;
mod stem;

use std::sync::OnceLock;

use time::{OffsetDateTime, format_description::well_known::Rfc3339};

pub use cortex::{
    emit_cortex_goal_forest_snapshot, emit_cortex_organ_request, emit_cortex_organ_response,
    emit_cortex_tick,
};
pub use spine::{
    emit_spine_adapter_lifecycle, emit_spine_dispatch_outcome, emit_spine_endpoint_lifecycle,
};
pub use stem::{
    emit_stem_descriptor_catalog, emit_stem_dispatch_transition, emit_stem_signal_transition,
};

pub(crate) use emit::emit_contract_event;

static RUN_ID: OnceLock<String> = OnceLock::new();
const FALLBACK_RUN_ID: &str = "run.uninitialized";

pub fn install_run_id(run_id: impl Into<String>) -> Result<(), String> {
    RUN_ID
        .set(run_id.into())
        .map_err(|_| "observability run id already installed".to_string())
}

fn current_run_id() -> &'static str {
    RUN_ID.get().map(String::as_str).unwrap_or(FALLBACK_RUN_ID)
}

fn timestamp_now() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}
