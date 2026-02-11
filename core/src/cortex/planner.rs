use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::{
    admission::types::{IntentAttempt, RequestedResources},
    cortex::{
        error::{CortexError, internal_error},
        noop::NoopGoalDecomposer,
        ports::GoalDecomposerPort,
        state::CortexState,
        types::{Goal, GoalClass, GoalScope, SchedulingContext},
    },
};

#[derive(Debug, Clone)]
pub struct PlannerConfig {
    pub base_survival_micro: i64,
    pub base_time_ms: u64,
    pub base_io_units: u64,
    pub base_token_units: u64,
}

impl Default for PlannerConfig {
    fn default() -> Self {
        Self {
            base_survival_micro: 120,
            base_time_ms: 100,
            base_io_units: 1,
            base_token_units: 96,
        }
    }
}

pub struct DeterministicPlanner {
    decomposer: Arc<dyn GoalDecomposerPort>,
    config: PlannerConfig,
}

impl Default for DeterministicPlanner {
    fn default() -> Self {
        Self {
            decomposer: Arc::new(NoopGoalDecomposer),
            config: PlannerConfig::default(),
        }
    }
}

impl DeterministicPlanner {
    pub fn new(decomposer: Arc<dyn GoalDecomposerPort>, config: PlannerConfig) -> Self {
        Self { decomposer, config }
    }

    pub fn plan(
        &self,
        state: &CortexState,
        scheduling: &[SchedulingContext],
    ) -> Result<Vec<IntentAttempt>, CortexError> {
        let mut sorted_scheduling = scheduling.to_vec();
        sorted_scheduling.sort_by(|lhs, rhs| {
            rhs.dynamic_priority
                .cmp(&lhs.dynamic_priority)
                .then_with(|| lhs.commitment_id.cmp(&rhs.commitment_id))
        });

        let mut attempts = Vec::with_capacity(sorted_scheduling.len());

        for (planner_slot, schedule) in sorted_scheduling.iter().enumerate() {
            let commitment = state
                .commitments
                .get(&schedule.commitment_id)
                .ok_or_else(|| {
                    internal_error(format!(
                        "commitment '{}' missing while planning",
                        schedule.commitment_id
                    ))
                })?;

            let goal = state.goals.get(&commitment.goal_id).ok_or_else(|| {
                internal_error(format!(
                    "goal '{}' missing while planning",
                    commitment.goal_id
                ))
            })?;

            let (affordance_key, capability_handle) = self.resolve_affordance(goal)?;

            let normalized_payload = canonicalize_json(&serde_json::json!({
                "goal_id": goal.id,
                "goal_title": goal.title,
                "goal_class": format!("{:?}", goal.class).to_lowercase(),
                "goal_scope": format!("{:?}", goal.scope).to_lowercase(),
                "cycle_id": state.cycle_id,
                "commitment_id": commitment.commitment_id,
            }));

            let requested_resources = RequestedResources {
                survival_micro: self
                    .config
                    .base_survival_micro
                    .saturating_add((goal.title.len() as i64) * 2)
                    .saturating_add((planner_slot as i64) * 3),
                time_ms: self
                    .config
                    .base_time_ms
                    .saturating_add((planner_slot as u64) * 20),
                io_units: self
                    .config
                    .base_io_units
                    .saturating_add((planner_slot as u64).min(3)),
                token_units: self
                    .config
                    .base_token_units
                    .saturating_add((goal.title.len() as u64) * 4),
            };

            let planner_slot = planner_slot as u16;
            let cost_attribution_id = derive_cost_attribution_id(
                state.cycle_id,
                &commitment.commitment_id,
                &goal.id,
                planner_slot,
            );

            let attempt_id = derive_attempt_id(
                state.cycle_id,
                &commitment.commitment_id,
                &goal.id,
                planner_slot,
                &affordance_key,
                &capability_handle,
                &normalized_payload,
                &requested_resources,
                &cost_attribution_id,
            );

            attempts.push(IntentAttempt {
                attempt_id,
                cycle_id: state.cycle_id,
                commitment_id: commitment.commitment_id.clone(),
                goal_id: goal.id.clone(),
                planner_slot,
                affordance_key,
                capability_handle,
                normalized_payload,
                requested_resources,
                cost_attribution_id,
            });
        }

        attempts.sort_by(|lhs, rhs| lhs.attempt_id.cmp(&rhs.attempt_id));
        Ok(attempts)
    }

    fn resolve_affordance(&self, goal: &Goal) -> Result<(String, String), CortexError> {
        if let Some(suggested) = self.decomposer.suggest_affordance(goal)? {
            return Ok(suggested);
        }

        let affordance_key = match goal.scope {
            GoalScope::Strategic => "deliberate.plan",
            GoalScope::Tactical => "execute.tool",
            GoalScope::Session => "observe.state",
        }
        .to_string();

        let capability_handle = match goal.class {
            GoalClass::Primary => "cap.core",
            GoalClass::Supporting => "cap.core",
            GoalClass::Exploratory => "cap.core.lite",
            GoalClass::Maintenance => "cap.core.minimal",
        }
        .to_string();

        Ok((affordance_key, capability_handle))
    }
}

pub fn derive_attempt_id(
    cycle_id: u64,
    commitment_id: &str,
    goal_id: &str,
    planner_slot: u16,
    affordance_key: &str,
    capability_handle: &str,
    normalized_payload: &serde_json::Value,
    requested_resources: &RequestedResources,
    cost_attribution_id: &str,
) -> String {
    let canonical = canonicalize_json(&serde_json::json!({
        "cycle_id": cycle_id,
        "commitment_id": commitment_id,
        "goal_id": goal_id,
        "planner_slot": planner_slot,
        "affordance_key": affordance_key,
        "capability_handle": capability_handle,
        "normalized_payload": normalized_payload,
        "requested_resources": {
            "survival_micro": requested_resources.survival_micro,
            "time_ms": requested_resources.time_ms,
            "io_units": requested_resources.io_units,
            "token_units": requested_resources.token_units,
        },
        "cost_attribution_id": cost_attribution_id,
    }));

    let mut hasher = Sha256::new();
    hasher.update(canonical.to_string().as_bytes());
    let digest = hasher.finalize();
    let hex = format!("{:x}", digest);
    format!("att:{}", &hex[..24])
}

pub fn derive_cost_attribution_id(
    cycle_id: u64,
    commitment_id: &str,
    goal_id: &str,
    planner_slot: u16,
) -> String {
    let canonical = canonicalize_json(&serde_json::json!({
        "cycle_id": cycle_id,
        "commitment_id": commitment_id,
        "goal_id": goal_id,
        "planner_slot": planner_slot,
    }));

    let mut hasher = Sha256::new();
    hasher.update(canonical.to_string().as_bytes());
    let digest = hasher.finalize();
    let hex = format!("{:x}", digest);
    format!("cat:{}", &hex[..24])
}

fn canonicalize_json(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut keys = map.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            let mut sorted = serde_json::Map::new();
            for key in keys {
                if let Some(item) = map.get(&key) {
                    sorted.insert(key, canonicalize_json(item));
                }
            }
            serde_json::Value::Object(sorted)
        }
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.iter().map(canonicalize_json).collect())
        }
        primitive => primitive.clone(),
    }
}
