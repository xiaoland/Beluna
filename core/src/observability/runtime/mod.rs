mod ai_gateway;
mod cortex;
mod spine;
mod stem;

use std::sync::OnceLock;

pub use crate::observability::owner_log::{
    AdapterLifecycleState, DescriptorCatalogChangeMode, DispatchOutcomeClass,
    EndpointLifecycleTransition, OrganResponseStatus,
};
pub use ai_gateway::{
    AiGatewayChatThreadArgs, AiGatewayChatTurnArgs, AiGatewayRequestArgs,
    emit_ai_gateway_chat_thread, emit_ai_gateway_chat_turn, emit_ai_gateway_request,
};
pub use cortex::{
    emit_cortex_goal_forest_patch, emit_cortex_goal_forest_snapshot, emit_cortex_organ_end,
    emit_cortex_organ_start,
};
pub use spine::{
    emit_spine_act_bind, emit_spine_act_outcome, emit_spine_adapter_lifecycle,
    emit_spine_endpoint_lifecycle, emit_spine_sense_ingress,
};
pub use stem::{
    emit_stem_afferent, emit_stem_afferent_rule, emit_stem_efferent, emit_stem_ns_catalog,
    emit_stem_proprioception, emit_stem_tick,
};

static RUN_ID: OnceLock<String> = OnceLock::new();
const FALLBACK_RUN_ID: &str = "run.uninitialized";

pub fn install_run_id(run_id: impl Into<String>) -> Result<(), String> {
    RUN_ID
        .set(run_id.into())
        .map_err(|_| "observability run id already installed".to_string())
}

pub(crate) fn current_run_id() -> &'static str {
    RUN_ID.get().map(String::as_str).unwrap_or(FALLBACK_RUN_ID)
}
