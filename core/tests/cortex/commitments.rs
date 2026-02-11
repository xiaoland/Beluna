use std::collections::BTreeMap;

use beluna::cortex::{
    CommitmentStatus, CortexCommand, CortexErrorKind, CortexFacade, Goal, GoalClass,
    GoalMetadataEntry, GoalMetadataValue, GoalScope, MetadataProvenance, MetadataSource,
};

fn sample_goal(id: &str) -> Goal {
    let mut metadata = BTreeMap::new();
    metadata.insert(
        "owner".to_string(),
        GoalMetadataEntry {
            value: GoalMetadataValue::Text("self".to_string()),
            provenance: MetadataProvenance {
                source: MetadataSource::Cortex,
                recorded_cycle: 0,
            },
        },
    );
    metadata.insert(
        "budget_hint".to_string(),
        GoalMetadataEntry {
            value: GoalMetadataValue::Integer(7),
            provenance: MetadataProvenance {
                source: MetadataSource::User,
                recorded_cycle: 0,
            },
        },
    );

    Goal {
        id: id.to_string(),
        title: "Ship deterministic admission".to_string(),
        class: GoalClass::Primary,
        scope: GoalScope::Strategic,
        parent_goal_id: None,
        metadata,
    }
}

#[test]
fn given_goal_and_commitment_when_created_then_goal_identity_and_commitment_lifecycle_are_separate()
{
    let mut facade = CortexFacade::with_defaults();

    facade
        .step(CortexCommand::ProposeGoal(sample_goal("g1")))
        .expect("goal proposal should work");
    facade
        .step(CortexCommand::CommitGoal {
            goal_id: "g1".to_string(),
            commitment_id: Some("c1".to_string()),
        })
        .expect("commitment creation should work");

    let state = facade.state();
    let goal = state.goals.get("g1").expect("goal should exist");
    let commitment = state
        .commitments
        .get("c1")
        .expect("commitment should exist");

    assert_eq!(goal.id, "g1");
    assert_eq!(goal.scope, GoalScope::Strategic);
    assert_eq!(commitment.goal_id, "g1");
    assert_eq!(commitment.status, CommitmentStatus::Active);
    assert!(commitment.created_cycle > 0);
}

#[test]
fn given_failed_commitment_without_failure_code_when_set_status_then_rejected() {
    let mut facade = CortexFacade::with_defaults();

    facade
        .step(CortexCommand::ProposeGoal(sample_goal("g1")))
        .expect("goal proposal should work");
    facade
        .step(CortexCommand::CommitGoal {
            goal_id: "g1".to_string(),
            commitment_id: Some("c1".to_string()),
        })
        .expect("commitment creation should work");

    let err = facade
        .step(CortexCommand::SetCommitmentStatus {
            commitment_id: "c1".to_string(),
            status: CommitmentStatus::Failed,
            superseded_by: None,
            failure_code: None,
        })
        .expect_err("failed status without failure_code must be rejected");

    assert_eq!(err.kind, CortexErrorKind::InvalidRequest);
}

#[test]
fn given_supersession_relation_when_status_is_updated_then_relation_is_recorded_without_new_status()
{
    let mut facade = CortexFacade::with_defaults();

    facade
        .step(CortexCommand::ProposeGoal(sample_goal("g1")))
        .expect("goal proposal should work");
    facade
        .step(CortexCommand::ProposeGoal(sample_goal("g2")))
        .expect("goal proposal should work");
    facade
        .step(CortexCommand::CommitGoal {
            goal_id: "g1".to_string(),
            commitment_id: Some("c1".to_string()),
        })
        .expect("commitment creation should work");
    facade
        .step(CortexCommand::CommitGoal {
            goal_id: "g2".to_string(),
            commitment_id: Some("c2".to_string()),
        })
        .expect("commitment creation should work");

    facade
        .step(CortexCommand::SetCommitmentStatus {
            commitment_id: "c1".to_string(),
            status: CommitmentStatus::Cancelled,
            superseded_by: Some("c2".to_string()),
            failure_code: None,
        })
        .expect("status update should succeed");

    let c1 = facade.state().commitments.get("c1").expect("c1 exists");
    assert_eq!(c1.status, CommitmentStatus::Cancelled);
    assert_eq!(c1.superseded_by.as_deref(), Some("c2"));
}
