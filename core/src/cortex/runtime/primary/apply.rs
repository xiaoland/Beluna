use crate::{
    cortex::{
        error::{CortexError, CortexErrorKind},
        types::CortexControlDirective,
    },
    observability::runtime as observability_runtime,
};

use super::{Cortex, attention::AttentionPhaseOutput, cleanup::CleanupPhaseOutput};

impl Cortex {
    pub(super) async fn apply_attention_result(
        &self,
        output: AttentionPhaseOutput,
    ) -> Result<CortexControlDirective, CortexError> {
        if let Some(rules) = output.gating_rules {
            let port = self.afferent_rule_control.as_ref().ok_or_else(|| {
                CortexError::new(
                    CortexErrorKind::Internal,
                    "afferent rule-control port is not configured",
                )
            })?;
            port.replace_ruleset(rules).await.map_err(|err| {
                CortexError::new(
                    CortexErrorKind::Internal,
                    format!("replace_afferent_ruleset_failed: {err}"),
                )
            })?;
        }

        Ok(CortexControlDirective {
            ignore_all_trigger_for_ticks: output.sleep_ticks,
        })
    }

    pub(super) async fn apply_cleanup_result(
        &self,
        cycle_id: u64,
        output: CleanupPhaseOutput,
    ) -> Result<(), CortexError> {
        let mut persisted_revision = None;
        if let Some(goal_forest_nodes) = output.patched_goal_forest.as_ref() {
            persisted_revision = Some(self.persist_goal_forest_nodes(goal_forest_nodes).await?);
        }

        if output.patched_goal_forest.is_some() || output.reset_context_requested {
            observability_runtime::emit_cortex_goal_forest_patch(
                cycle_id,
                &format!("cortex.cleanup.apply:{cycle_id}"),
                None,
                Some(serde_json::json!({
                    "patched_goal_forest": output.patched_goal_forest,
                    "reset_context_requested": output.reset_context_requested,
                })),
                persisted_revision,
                Some(output.reset_context_requested),
                None,
            );
        }

        if output.reset_context_requested {
            self.reset_primary_thread_state("cleanup_reset_context")
                .await;
        }

        Ok(())
    }
}
