use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use tokio::time::{Duration, Instant};

use beluna::{
    cortex::{
        ReactionLimits,
        testing::{
            TestActsHelperOutput, TestGoalStackPatch, TestGoalStackPatchOp, TestHooks, boxed,
            cortex_with_hooks,
        },
    },
    types::{
        CognitionState, NeuralSignalDescriptor, NeuralSignalDescriptorCatalog, NeuralSignalType,
        PhysicalLedgerSnapshot, PhysicalState, Sense, SenseDatum,
    },
};

fn output_ir() -> String {
    "<output-ir><acts>body</acts><goal-stack-patch>body</goal-stack-patch></output-ir>".to_string()
}

fn physical_state_with_descriptor(descriptor_id: &str) -> PhysicalState {
    PhysicalState {
        cycle_id: 1,
        ledger: PhysicalLedgerSnapshot::default(),
        capabilities: NeuralSignalDescriptorCatalog {
            version: "v1".to_string(),
            entries: vec![NeuralSignalDescriptor {
                r#type: NeuralSignalType::Act,
                endpoint_id: "ep.demo".to_string(),
                neural_signal_descriptor_id: descriptor_id.to_string(),
                payload_schema: serde_json::json!({"type":"object"}),
            }],
        },
    }
}

fn domain_sense() -> Sense {
    Sense::Domain(SenseDatum {
        sense_id: "sense:1".to_string(),
        endpoint_id: "ep.demo".to_string(),
        neural_signal_descriptor_id: "sense.demo".to_string(),
        payload: serde_json::json!({"x":1}),
    })
}

#[tokio::test]
async fn act_descriptor_helper_cache_hits_when_descriptor_input_unchanged() {
    let calls = Arc::new(AtomicUsize::new(0));
    let sense_helper = Arc::new(|_req| boxed(async { Ok("senses".to_string()) }));
    let act_descriptor_helper = Arc::new({
        let calls = Arc::clone(&calls);
        move |_req| {
            let calls = Arc::clone(&calls);
            boxed(async move {
                calls.fetch_add(1, Ordering::Relaxed);
                Ok("catalog".to_string())
            })
        }
    });
    let primary = Arc::new(|_req| boxed(async move { Ok(output_ir()) }));
    let acts_helper = Arc::new(|_req| boxed(async { Ok(TestActsHelperOutput::default()) }));
    let goal_stack_helper = Arc::new(|_req| boxed(async { Ok(TestGoalStackPatch::default()) }));

    let cortex = cortex_with_hooks(
        TestHooks::new(
            sense_helper,
            act_descriptor_helper,
            primary,
            acts_helper,
            goal_stack_helper,
        ),
        ReactionLimits::default(),
    );

    let state = physical_state_with_descriptor("cap.demo");
    cortex
        .cortex(&[domain_sense()], &state, &CognitionState::default())
        .await
        .expect("first run should succeed");
    cortex
        .cortex(&[domain_sense()], &state, &CognitionState::default())
        .await
        .expect("second run should succeed");

    assert_eq!(calls.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn act_descriptor_helper_cache_misses_when_descriptor_input_changes() {
    let calls = Arc::new(AtomicUsize::new(0));
    let sense_helper = Arc::new(|_req| boxed(async { Ok("senses".to_string()) }));
    let act_descriptor_helper = Arc::new({
        let calls = Arc::clone(&calls);
        move |_req| {
            let calls = Arc::clone(&calls);
            boxed(async move {
                calls.fetch_add(1, Ordering::Relaxed);
                Ok("catalog".to_string())
            })
        }
    });
    let primary = Arc::new(|_req| boxed(async move { Ok(output_ir()) }));
    let acts_helper = Arc::new(|_req| boxed(async { Ok(TestActsHelperOutput::default()) }));
    let goal_stack_helper = Arc::new(|_req| boxed(async { Ok(TestGoalStackPatch::default()) }));

    let cortex = cortex_with_hooks(
        TestHooks::new(
            sense_helper,
            act_descriptor_helper,
            primary,
            acts_helper,
            goal_stack_helper,
        ),
        ReactionLimits::default(),
    );

    cortex
        .cortex(
            &[domain_sense()],
            &physical_state_with_descriptor("cap.demo"),
            &CognitionState::default(),
        )
        .await
        .expect("first run should succeed");
    cortex
        .cortex(
            &[domain_sense()],
            &physical_state_with_descriptor("cap.demo.v2"),
            &CognitionState::default(),
        )
        .await
        .expect("second run should succeed");

    assert_eq!(calls.load(Ordering::Relaxed), 2);
}

#[tokio::test]
async fn goal_stack_patch_ops_are_applied_in_order() {
    let sense_helper = Arc::new(|_req| boxed(async { Ok("senses".to_string()) }));
    let act_descriptor_helper = Arc::new(|_req| boxed(async { Ok("catalog".to_string()) }));
    let primary = Arc::new(|_req| boxed(async move { Ok(output_ir()) }));
    let acts_helper = Arc::new(|_req| boxed(async { Ok(TestActsHelperOutput::default()) }));
    let goal_stack_helper = Arc::new(|_req| {
        boxed(async {
            Ok(TestGoalStackPatch {
                ops: vec![
                    TestGoalStackPatchOp::Push {
                        goal_id: "g1".to_string(),
                        summary: "s1".to_string(),
                    },
                    TestGoalStackPatchOp::ReplaceTop {
                        goal_id: "g1b".to_string(),
                        summary: "s1b".to_string(),
                    },
                    TestGoalStackPatchOp::Push {
                        goal_id: "g2".to_string(),
                        summary: "s2".to_string(),
                    },
                    TestGoalStackPatchOp::Pop,
                    TestGoalStackPatchOp::Clear,
                ],
            })
        })
    });

    let cortex = cortex_with_hooks(
        TestHooks::new(
            sense_helper,
            act_descriptor_helper,
            primary,
            acts_helper,
            goal_stack_helper,
        ),
        ReactionLimits::default(),
    );

    let output = cortex
        .cortex(
            &[domain_sense()],
            &physical_state_with_descriptor("cap.demo"),
            &CognitionState::default(),
        )
        .await
        .expect("run should succeed");

    assert_eq!(output.new_cognition_state.revision, 1);
    assert!(output.new_cognition_state.goal_stack.is_empty());
}

#[tokio::test]
async fn input_helpers_run_concurrently() {
    let sense_helper = Arc::new(|_req| {
        boxed(async {
            tokio::time::sleep(Duration::from_millis(120)).await;
            Ok("senses".to_string())
        })
    });
    let act_descriptor_helper = Arc::new(|_req| {
        boxed(async {
            tokio::time::sleep(Duration::from_millis(120)).await;
            Ok("catalog".to_string())
        })
    });
    let primary = Arc::new(|_req| boxed(async { Ok(output_ir()) }));
    let acts_helper = Arc::new(|_req| boxed(async { Ok(TestActsHelperOutput::default()) }));
    let goal_stack_helper = Arc::new(|_req| boxed(async { Ok(TestGoalStackPatch::default()) }));
    let cortex = cortex_with_hooks(
        TestHooks::new(
            sense_helper,
            act_descriptor_helper,
            primary,
            acts_helper,
            goal_stack_helper,
        ),
        ReactionLimits::default(),
    );

    let started = Instant::now();
    cortex
        .cortex(
            &[domain_sense()],
            &physical_state_with_descriptor("cap.demo"),
            &CognitionState::default(),
        )
        .await
        .expect("run should succeed");
    let elapsed = started.elapsed();

    assert!(elapsed < Duration::from_millis(230), "elapsed={elapsed:?}");
}

#[tokio::test]
async fn output_helpers_run_concurrently() {
    let sense_helper = Arc::new(|_req| boxed(async { Ok("senses".to_string()) }));
    let act_descriptor_helper = Arc::new(|_req| boxed(async { Ok("catalog".to_string()) }));
    let primary = Arc::new(|_req| boxed(async { Ok(output_ir()) }));
    let acts_helper = Arc::new(|_req| {
        boxed(async {
            tokio::time::sleep(Duration::from_millis(120)).await;
            Ok(TestActsHelperOutput::default())
        })
    });
    let goal_stack_helper = Arc::new(|_req| {
        boxed(async {
            tokio::time::sleep(Duration::from_millis(120)).await;
            Ok(TestGoalStackPatch::default())
        })
    });
    let cortex = cortex_with_hooks(
        TestHooks::new(
            sense_helper,
            act_descriptor_helper,
            primary,
            acts_helper,
            goal_stack_helper,
        ),
        ReactionLimits::default(),
    );

    let started = Instant::now();
    cortex
        .cortex(
            &[domain_sense()],
            &physical_state_with_descriptor("cap.demo"),
            &CognitionState::default(),
        )
        .await
        .expect("run should succeed");
    let elapsed = started.elapsed();

    assert!(elapsed < Duration::from_millis(230), "elapsed={elapsed:?}");
}
