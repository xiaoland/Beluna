use std::collections::BTreeMap;

use beluna::mind::{
    ConflictResolution, ConflictResolver, DelegationCoordinatorPort, DeterministicConflictResolver,
    DeterministicEvaluator, DeterministicEvolutionDecider, DeterministicPreemptionDecider,
    DeterministicSafePointPolicy, EvolutionDecider, Goal, MindCommand, MindDecision, MindFacade,
    MindState, NoopMemoryPolicy, NormativeEvaluator, PreemptionDecider, SafePointPolicy,
    error::MindError,
    ports::MemoryPolicyPort,
    types::{ActionIntent, GoalId, IntentKind},
};

struct StaticDelegationPlanner;

impl DelegationCoordinatorPort for StaticDelegationPlanner {
    fn plan(
        &self,
        _state: &MindState,
        goal_id: Option<&GoalId>,
    ) -> Result<Vec<ActionIntent>, MindError> {
        Ok(vec![ActionIntent {
            intent_id: "intent-1".to_string(),
            goal_id: goal_id.cloned(),
            kind: IntentKind::Delegate,
            description: "delegate once".to_string(),
            checkpoint_token: None,
            metadata: BTreeMap::new(),
        }])
    }
}

struct StaticConflictResolver;

impl ConflictResolver for StaticConflictResolver {
    fn resolve(
        &self,
        _cases: &[beluna::mind::ConflictCase],
    ) -> Result<Vec<ConflictResolution>, MindError> {
        Ok(vec![ConflictResolution::NoConflict])
    }
}

fn build_goal(id: &str, priority: u8, parent: Option<&str>) -> Goal {
    Goal {
        id: id.to_string(),
        title: format!("goal-{id}"),
        level: beluna::mind::GoalLevel::High,
        parent_goal_id: parent.map(str::to_string),
        priority,
        created_cycle: 0,
        metadata: BTreeMap::new(),
    }
}

#[test]
fn given_same_input_and_state_when_step_twice_then_outputs_are_identical() {
    let mut facade_a = MindFacade::with_defaults();
    let mut facade_b = MindFacade::with_defaults();
    let command = MindCommand::EvaluateNow;

    let output_a = facade_a.step(command.clone()).expect("step should succeed");
    let output_b = facade_b.step(command).expect("step should succeed");

    assert_eq!(output_a, output_b);
}

#[test]
fn given_new_goal_with_active_goal_when_step_then_preemption_before_delegation() {
    let mut facade = MindFacade::new(
        MindState::default(),
        Box::new(DeterministicSafePointPolicy) as Box<dyn SafePointPolicy>,
        Box::new(DeterministicPreemptionDecider::default()) as Box<dyn PreemptionDecider>,
        Box::new(StaticDelegationPlanner) as Box<dyn DelegationCoordinatorPort>,
        Box::new(DeterministicEvaluator) as Box<dyn NormativeEvaluator>,
        Box::new(DeterministicConflictResolver) as Box<dyn ConflictResolver>,
        Box::new(NoopMemoryPolicy) as Box<dyn MemoryPolicyPort>,
        Box::new(DeterministicEvolutionDecider::default()) as Box<dyn EvolutionDecider>,
    );

    facade
        .step(MindCommand::ProposeGoal(build_goal("g1", 1, None)))
        .expect("first step should succeed");

    let output = facade
        .step(MindCommand::ProposeGoal(build_goal("g2", 9, Some("other"))))
        .expect("second step should succeed");

    let preemption_index = output
        .decisions
        .iter()
        .position(|decision| matches!(decision, MindDecision::Preemption(_)))
        .expect("preemption decision should exist");

    let delegation_index = output
        .decisions
        .iter()
        .position(|decision| matches!(decision, MindDecision::DelegationPlan(_)))
        .expect("delegation plan should exist");

    assert!(preemption_index < delegation_index);
}

#[test]
fn given_evaluation_report_when_step_then_conflict_resolution_before_evolution() {
    let mut facade = MindFacade::new(
        MindState::default(),
        Box::new(DeterministicSafePointPolicy),
        Box::new(DeterministicPreemptionDecider::default()),
        Box::new(StaticDelegationPlanner),
        Box::new(DeterministicEvaluator),
        Box::new(StaticConflictResolver),
        Box::new(NoopMemoryPolicy),
        Box::new(DeterministicEvolutionDecider::default()),
    );

    let output = facade
        .step(MindCommand::EvaluateNow)
        .expect("step should succeed");

    let conflict_index = output
        .decisions
        .iter()
        .position(|decision| matches!(decision, MindDecision::Conflict(_)))
        .expect("conflict decision should exist");

    let evolution_index = output
        .decisions
        .iter()
        .position(|decision| matches!(decision, MindDecision::Evolution(_)))
        .expect("evolution decision should exist");

    assert!(conflict_index < evolution_index);
}

#[test]
fn given_noop_ports_when_step_then_loop_completes_without_external_side_effect() {
    let mut facade = MindFacade::with_defaults();

    let output = facade
        .step(MindCommand::EvaluateNow)
        .expect("step should succeed");

    assert!(output.decisions.iter().any(|decision| matches!(
        decision,
        MindDecision::MemoryPolicy(beluna::mind::MemoryDirective::KeepTransient)
    )));
    assert!(facade.state().pending_intents.is_empty());
}
