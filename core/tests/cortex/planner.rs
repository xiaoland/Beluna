use beluna::cortex::{CortexCommand, CortexFacade, Goal, GoalClass, GoalScope, derive_attempt_id};
use beluna::non_cortex::types::RequestedResources;

fn goal(id: &str) -> Goal {
    Goal {
        id: id.to_string(),
        title: "Stabilize boundary contracts".to_string(),
        class: GoalClass::Primary,
        scope: GoalScope::Strategic,
        parent_goal_id: None,
        metadata: Default::default(),
    }
}

#[test]
fn given_equivalent_inputs_when_derive_attempt_id_then_output_is_stable() {
    let payload_a = serde_json::json!({"b": 2, "a": 1});
    let payload_b = serde_json::json!({"a": 1, "b": 2});
    let resources = RequestedResources {
        survival_micro: 200,
        time_ms: 100,
        io_units: 1,
        token_units: 50,
    };

    let lhs = derive_attempt_id(
        10,
        "c1",
        "g1",
        0,
        "deliberate.plan",
        "cap.core",
        &payload_a,
        &resources,
        "cat:123",
    );
    let rhs = derive_attempt_id(
        10,
        "c1",
        "g1",
        0,
        "deliberate.plan",
        "cap.core",
        &payload_b,
        &resources,
        "cat:123",
    );

    assert_eq!(lhs, rhs);
}

#[test]
fn given_identical_state_when_planning_then_attempts_are_deterministic() {
    let mut a = CortexFacade::with_defaults();
    let mut b = CortexFacade::with_defaults();

    let sequence = vec![
        CortexCommand::ProposeGoal(goal("g1")),
        CortexCommand::CommitGoal {
            goal_id: "g1".to_string(),
            commitment_id: Some("c1".to_string()),
        },
        CortexCommand::PlanNow,
    ];

    let mut out_a = None;
    let mut out_b = None;
    for command in &sequence {
        out_a = Some(
            a.step(command.clone())
                .expect("sequence should execute for facade a"),
        );
        out_b = Some(
            b.step(command.clone())
                .expect("sequence should execute for facade b"),
        );
    }

    let out_a = out_a.expect("output exists");
    let out_b = out_b.expect("output exists");

    assert_eq!(out_a.attempts, out_b.attempts);
    assert!(
        out_a
            .attempts
            .windows(2)
            .all(|window| window[0].attempt_id <= window[1].attempt_id)
    );
}
