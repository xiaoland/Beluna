use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::cortex::{
    cognition::{GoalNode, GoalTree},
    error::CortexError,
    helpers::{self, CognitionOrgan, HelperRuntime},
    prompts,
    testing::GoalTreeHelperRequest as TestGoalTreeHelperRequest,
};

const GOAL_TREE_EMPTY_PURSUITS_ONE_SHOT: &str = concat!(
    "1 (w=0.60) Build an environment baseline from current senses and constraints.\n",
    "1.1 (w=0.50) Verify executable acts and their payload boundaries.\n",
    "1.2 (w=0.40) Extract high-signal observations and unresolved unknowns.\n",
    "2 (w=0.50) Preserve compact cross-tick heuristics.\n",
    "2.1 (w=0.40) Keep focal-awareness as styleless bullet point statements."
);

const L1_MEMORY_EMPTY_ONE_SHOT: &str = concat!(
    "- Record concrete observations, not narrative.\n",
    "- Keep one bullet per actionable heuristic.\n",
    "- Prefer short statements that can guide the next tick directly."
);

#[derive(Clone, Default)]
pub(crate) struct GoalTreeInputHelper {
    cache: Arc<RwLock<HashMap<String, String>>>,
}

#[derive(Debug, Clone)]
pub(crate) struct GoalTreeInputSections {
    pub instincts_section: String,
    pub willpower_matrix_section: String,
}

impl GoalTreeInputHelper {
    pub(crate) async fn to_input_ir_sections(
        &self,
        runtime: &impl HelperRuntime,
        cycle_id: u64,
        goal_tree: &GoalTree,
    ) -> Result<GoalTreeInputSections, CortexError> {
        let instincts_section = instincts_section(&goal_tree.root_partition);
        let willpower_matrix_section = self
            .to_willpower_matrix_section(runtime, cycle_id, &goal_tree.user_partition)
            .await?;
        Ok(GoalTreeInputSections {
            instincts_section,
            willpower_matrix_section,
        })
    }

    async fn to_willpower_matrix_section(
        &self,
        runtime: &impl HelperRuntime,
        cycle_id: u64,
        user_partition: &[GoalNode],
    ) -> Result<String, CortexError> {
        let stage = CognitionOrgan::GoalTree.stage();
        let user_partition_json = goal_tree_user_partition_json(user_partition);
        let input_payload = helpers::pretty_json(&serde_json::json!({
            "goal_tree_user_partition": user_partition,
        }));
        helpers::log_organ_input(cycle_id, stage, &input_payload);

        if user_partition.is_empty() {
            let output = goal_tree_empty_pursuits_one_shot().to_string();
            helpers::log_organ_output(cycle_id, stage, &output);
            return Ok(output);
        }

        let cache_key = goal_tree_cache_key(user_partition);
        if let Some(cached) = self.get_cached_section(&cache_key).await {
            tracing::debug!(
                target: "cortex",
                cycle_id = cycle_id,
                cache_key = %cache_key,
                "goal_tree_helper_cache_hit"
            );
            helpers::log_organ_output(cycle_id, stage, &cached);
            return Ok(cached);
        }

        let generated = if let Some(hooks) = runtime.hooks() {
            (hooks.goal_tree_helper)(TestGoalTreeHelperRequest {
                cycle_id,
                user_partition_json: user_partition_json.clone(),
            })
            .await?
        } else {
            let prompt = prompts::build_goal_tree_helper_prompt(&user_partition_json);
            runtime
                .run_text_organ_with_system(
                    cycle_id,
                    CognitionOrgan::GoalTree,
                    runtime.limits().max_sub_output_tokens,
                    prompts::goal_tree_helper_system_prompt(),
                    prompt,
                )
                .await?
        };

        self.cache_section(cache_key, generated.clone()).await;
        helpers::log_organ_output(cycle_id, stage, &generated);
        Ok(generated)
    }

    async fn get_cached_section(&self, cache_key: &str) -> Option<String> {
        self.cache.read().await.get(cache_key).cloned()
    }

    async fn cache_section(&self, cache_key: String, value: String) {
        self.cache.write().await.insert(cache_key, value);
    }
}

pub(crate) fn fallback_goal_tree_section(user_partition: &[GoalNode]) -> String {
    serde_json::to_string_pretty(user_partition).unwrap_or_else(|_| "[]".to_string())
}

pub(crate) fn fallback_input_ir_sections(goal_tree: &GoalTree) -> GoalTreeInputSections {
    GoalTreeInputSections {
        instincts_section: instincts_section(&goal_tree.root_partition),
        willpower_matrix_section: fallback_goal_tree_section(&goal_tree.user_partition),
    }
}

pub(crate) fn instincts_section(root_partition: &[String]) -> String {
    serde_json::to_string_pretty(root_partition).unwrap_or_else(|_| "[]".to_string())
}

pub(crate) fn l1_memory_section(l1_memory: &[String]) -> String {
    if l1_memory.is_empty() {
        return l1_memory_empty_one_shot().to_string();
    }
    serde_json::to_string_pretty(l1_memory).unwrap_or_else(|_| "[]".to_string())
}

pub(crate) fn goal_tree_user_partition_json(user_partition: &[GoalNode]) -> String {
    serde_json::to_string_pretty(user_partition).unwrap_or_else(|_| "[]".to_string())
}

pub(crate) fn l1_memory_json(l1_memory: &[String]) -> String {
    serde_json::to_string_pretty(l1_memory).unwrap_or_else(|_| "[]".to_string())
}

fn goal_tree_cache_key(user_partition: &[GoalNode]) -> String {
    let canonical = serde_json::to_string(user_partition).unwrap_or_else(|_| "[]".to_string());
    format!("{:x}", md5::compute(canonical.as_bytes()))
}

pub(crate) fn goal_tree_empty_pursuits_one_shot() -> &'static str {
    GOAL_TREE_EMPTY_PURSUITS_ONE_SHOT
}

pub(crate) fn l1_memory_empty_one_shot() -> &'static str {
    L1_MEMORY_EMPTY_ONE_SHOT
}
