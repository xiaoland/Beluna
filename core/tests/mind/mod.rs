mod conflict;
mod evaluator;
mod evolution;
mod facade_loop;
mod goal_manager;
mod preemption;

use std::collections::BTreeMap;

use beluna::mind::{Goal, GoalLevel};

pub fn high_goal(id: &str, priority: u8) -> Goal {
    Goal {
        id: id.to_string(),
        title: format!("goal-{id}"),
        level: GoalLevel::High,
        parent_goal_id: None,
        priority,
        created_cycle: 0,
        metadata: BTreeMap::new(),
    }
}

pub fn mid_goal(id: &str, parent: &str, priority: u8) -> Goal {
    Goal {
        id: id.to_string(),
        title: format!("goal-{id}"),
        level: GoalLevel::Mid,
        parent_goal_id: Some(parent.to_string()),
        priority,
        created_cycle: 0,
        metadata: BTreeMap::new(),
    }
}
