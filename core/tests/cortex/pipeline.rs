use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use beluna::{
    cortex::{
        Cortex, CortexErrorKind, ReactionLimits,
        testing::{
            TestActDraft, TestActsHelperOutput, TestGoalStackPatch, TestGoalStackPatchOp,
            TestHooks, boxed, cortex_with_hooks,
        },
    },
    types::{
        CognitionState, NeuralSignalDescriptor, NeuralSignalDescriptorCatalog, NeuralSignalType,
        PhysicalLedgerSnapshot, PhysicalState, Sense, SenseDatum, is_uuid_v7,
    },
};

fn base_physical_state() -> PhysicalState {
    PhysicalState {
        cycle_id: 1,
        ledger: PhysicalLedgerSnapshot::default(),
        capabilities: NeuralSignalDescriptorCatalog {
            version: "v1".to_string(),
            entries: vec![NeuralSignalDescriptor {
                r#type: NeuralSignalType::Act,
                endpoint_id: "ep.demo".to_string(),
                neural_signal_descriptor_id: "cap.demo".to_string(),
                payload_schema: serde_json::json!({"type":"object"}),
            }],
        },
    }
}

fn valid_output_ir() -> String {
    "<output-ir><acts>markdown acts body</acts><goal-stack-patch>markdown patch body</goal-stack-patch></output-ir>".to_string()
}

fn build_pipeline(
    primary_output: String,
    acts: Vec<TestActDraft>,
    patch: TestGoalStackPatch,
) -> Cortex {
    let sense_helper = Arc::new(|_req| boxed(async { Ok("senses section".to_string()) }));
    let act_descriptor_helper =
        Arc::new(|_req| boxed(async { Ok("act descriptor section".to_string()) }));
    let primary = Arc::new(move |_req| {
        let primary_output = primary_output.clone();
        boxed(async move { Ok(primary_output) })
    });
    let acts_helper = Arc::new(move |_req| {
        let acts = acts.clone();
        boxed(async move { Ok(TestActsHelperOutput { acts }) })
    });
    let goal_stack_helper = Arc::new(move |_req| {
        let patch = patch.clone();
        boxed(async move { Ok(patch) })
    });

    cortex_with_hooks(
        TestHooks::new(
            sense_helper,
            act_descriptor_helper,
            primary,
            acts_helper,
            goal_stack_helper,
        ),
        ReactionLimits::default(),
    )
}

#[tokio::test]
async fn cortex_pipeline_emits_acts_and_new_cognition() {
    let pipeline = build_pipeline(
        valid_output_ir(),
        vec![TestActDraft {
            endpoint_id: "ep.demo".to_string(),
            neural_signal_descriptor_id: "cap.demo".to_string(),
            payload: serde_json::json!({"ok":true}),
        }],
        TestGoalStackPatch {
            ops: vec![TestGoalStackPatchOp::Push {
                goal_id: "goal:1".to_string(),
                summary: "ship".to_string(),
            }],
        },
    );

    let output = pipeline
        .cortex(
            &[Sense::Domain(SenseDatum {
                sense_id: "sense:1".to_string(),
                endpoint_id: "ep.demo".to_string(),
                neural_signal_descriptor_id: "sense.demo".to_string(),
                payload: serde_json::json!({"x":1}),
            })],
            &base_physical_state(),
            &CognitionState::default(),
        )
        .await
        .expect("pipeline should succeed");

    assert_eq!(output.acts.len(), 1);
    assert!(is_uuid_v7(&output.acts[0].act_id));
    assert_eq!(output.new_cognition_state.revision, 1);
    assert_eq!(output.new_cognition_state.goal_stack.len(), 1);
}

#[tokio::test]
async fn cortex_pipeline_rejects_sleep_sense() {
    let pipeline = build_pipeline(valid_output_ir(), Vec::new(), TestGoalStackPatch::default());

    let err = pipeline
        .cortex(
            &[Sense::Sleep],
            &base_physical_state(),
            &CognitionState::default(),
        )
        .await
        .expect_err("sleep should not reach cortex");
    assert_eq!(err.kind, CortexErrorKind::InvalidReactionInput);
}

#[tokio::test]
async fn cortex_pipeline_primary_failure_returns_noop_without_cognition_change() {
    let sense_helper = Arc::new(|_req| boxed(async { Ok("senses section".to_string()) }));
    let act_descriptor_helper =
        Arc::new(|_req| boxed(async { Ok("act descriptor section".to_string()) }));
    let primary = Arc::new(|_req| {
        boxed(async {
            Err(beluna::cortex::CortexError::new(
                CortexErrorKind::PrimaryInferenceFailed,
                "boom",
            ))
        })
    });
    let acts_helper = Arc::new(|_req| boxed(async { Ok(TestActsHelperOutput::default()) }));
    let goal_stack_helper = Arc::new(|_req| boxed(async { Ok(TestGoalStackPatch::default()) }));
    let pipeline = cortex_with_hooks(
        TestHooks::new(
            sense_helper,
            act_descriptor_helper,
            primary,
            acts_helper,
            goal_stack_helper,
        ),
        ReactionLimits::default(),
    );
    let cognition = CognitionState::default();

    let output = pipeline
        .cortex(
            &[Sense::Domain(SenseDatum {
                sense_id: "sense:1".to_string(),
                endpoint_id: "ep.demo".to_string(),
                neural_signal_descriptor_id: "sense.demo".to_string(),
                payload: serde_json::json!({"x":1}),
            })],
            &base_physical_state(),
            &cognition,
        )
        .await
        .expect("primary failure should degrade to noop");

    assert!(output.acts.is_empty());
    assert_eq!(output.new_cognition_state, cognition);
}

#[tokio::test]
async fn cortex_pipeline_invalid_output_ir_contract_returns_noop() {
    let pipeline = build_pipeline(
        "<acts>missing root</acts>".to_string(),
        vec![TestActDraft {
            endpoint_id: "ep.demo".to_string(),
            neural_signal_descriptor_id: "cap.demo".to_string(),
            payload: serde_json::json!({"ok":true}),
        }],
        TestGoalStackPatch::default(),
    );
    let cognition = CognitionState::default();

    let output = pipeline
        .cortex(
            &[Sense::Domain(SenseDatum {
                sense_id: "sense:1".to_string(),
                endpoint_id: "ep.demo".to_string(),
                neural_signal_descriptor_id: "sense.demo".to_string(),
                payload: serde_json::json!({"x":1}),
            })],
            &base_physical_state(),
            &cognition,
        )
        .await
        .expect("invalid primary output should degrade to noop");

    assert!(output.acts.is_empty());
    assert_eq!(output.new_cognition_state, cognition);
}

#[tokio::test]
async fn input_ir_contains_root_tag() {
    let has_root = Arc::new(AtomicBool::new(false));
    let sense_helper = Arc::new(|_req| boxed(async { Ok("senses section".to_string()) }));
    let act_descriptor_helper =
        Arc::new(|_req| boxed(async { Ok("act descriptor section".to_string()) }));
    let primary = Arc::new({
        let has_root = Arc::clone(&has_root);
        move |req: beluna::cortex::testing::PrimaryRequest| {
            let has_root = Arc::clone(&has_root);
            boxed(async move {
                has_root.store(req.input_ir.contains("<input-ir>"), Ordering::Relaxed);
                Ok(valid_output_ir())
            })
        }
    });
    let acts_helper = Arc::new(|_req| boxed(async { Ok(TestActsHelperOutput::default()) }));
    let goal_stack_helper = Arc::new(|_req| boxed(async { Ok(TestGoalStackPatch::default()) }));
    let pipeline = cortex_with_hooks(
        TestHooks::new(
            sense_helper,
            act_descriptor_helper,
            primary,
            acts_helper,
            goal_stack_helper,
        ),
        ReactionLimits::default(),
    );

    pipeline
        .cortex(
            &[Sense::Domain(SenseDatum {
                sense_id: "sense:1".to_string(),
                endpoint_id: "ep.demo".to_string(),
                neural_signal_descriptor_id: "sense.demo".to_string(),
                payload: serde_json::json!({"x":1}),
            })],
            &base_physical_state(),
            &CognitionState::default(),
        )
        .await
        .expect("pipeline should succeed");

    assert!(has_root.load(Ordering::Relaxed));
}
