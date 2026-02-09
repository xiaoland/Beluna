use beluna::mind::{Goal, GoalManager, GoalStatus, MindState, error::MindErrorKind};

use super::{high_goal, mid_goal};

#[test]
fn given_no_active_goal_when_activate_then_goal_becomes_active() {
    let mut state = MindState::default();
    let goal = high_goal("g1", 1);

    GoalManager::register_goal(&mut state, goal.clone()).expect("goal should register");
    GoalManager::activate_goal(&mut state, &goal.id).expect("goal should activate");

    assert_eq!(state.active_goal_id, Some(goal.id.clone()));
    let record = state.goals.get(&goal.id).expect("goal should exist");
    assert_eq!(record.status, GoalStatus::Active);
}

#[test]
fn given_active_goal_when_activate_other_without_preemption_then_rejected() {
    let mut state = MindState::default();
    let g1 = high_goal("g1", 1);
    let g2 = high_goal("g2", 2);

    GoalManager::register_goal(&mut state, g1.clone()).expect("goal should register");
    GoalManager::register_goal(&mut state, g2.clone()).expect("goal should register");
    GoalManager::activate_goal(&mut state, &g1.id).expect("goal should activate");

    let err = GoalManager::activate_goal(&mut state, &g2.id).expect_err("must reject activation");
    assert_eq!(err.kind, MindErrorKind::PolicyViolation);
}

#[test]
fn given_merged_goal_when_reactivate_then_rejected() {
    let mut state = MindState::default();
    let g1 = high_goal("g1", 1);
    let g2 = high_goal("g2", 2);
    let merged = Goal {
        id: "gm".to_string(),
        title: "merged".to_string(),
        level: beluna::mind::GoalLevel::High,
        parent_goal_id: None,
        priority: 2,
        created_cycle: 0,
        metadata: Default::default(),
    };

    GoalManager::register_goal(&mut state, g1.clone()).expect("goal should register");
    GoalManager::register_goal(&mut state, g2.clone()).expect("goal should register");
    GoalManager::activate_goal(&mut state, &g1.id).expect("goal should activate");
    GoalManager::merge_goals(&mut state, &g1.id, &g2.id, merged).expect("merge should succeed");

    let err = GoalManager::activate_goal(&mut state, &g1.id).expect_err("reactivation must fail");
    assert_eq!(err.kind, MindErrorKind::PolicyViolation);
}

#[test]
fn given_invalid_parent_goal_when_register_then_invalid_request() {
    let mut state = MindState::default();
    let child = mid_goal("child", "missing-parent", 1);

    let err =
        GoalManager::register_goal(&mut state, child).expect_err("must reject unknown parent");
    assert_eq!(err.kind, MindErrorKind::InvalidRequest);
}
