use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use tokio::sync::{Mutex, mpsc};

use beluna::{
    afferent_pathway::SenseAfferentPathway,
    config::SpineRuntimeConfig,
    continuity::ContinuityEngine,
    cortex::{
        Cortex, ReactionLimits,
        testing::{TestActsHelperOutput, TestGoalStackPatch, TestHooks, boxed, cortex_with_hooks},
    },
    ledger::LedgerStage,
    spine::Spine,
    stem::Stem,
    types::{Sense, SenseDatum},
};

fn test_spine() -> Arc<Spine> {
    let config = SpineRuntimeConfig { adapters: vec![] };
    Spine::new(&config, SenseAfferentPathway::new(4).0)
}

fn valid_output_ir() -> String {
    "<output-ir><acts>body</acts><goal-stack-patch>body</goal-stack-patch></output-ir>".to_string()
}

fn counting_cortex(calls: Arc<AtomicUsize>, batch_sizes: Arc<Mutex<Vec<usize>>>) -> Arc<Cortex> {
    let sense_helper = Arc::new(|_req| boxed(async { Ok("senses".to_string()) }));
    let act_descriptor_helper = Arc::new(|_req| boxed(async { Ok("catalog".to_string()) }));
    let primary = Arc::new(move |req: beluna::cortex::testing::PrimaryRequest| {
        let calls = Arc::clone(&calls);
        let batch_sizes = Arc::clone(&batch_sizes);
        boxed(async move {
            calls.fetch_add(1, Ordering::Relaxed);
            batch_sizes.lock().await.push(req.senses.len());
            Ok(valid_output_ir())
        })
    });
    let acts_helper = Arc::new(|_req| boxed(async { Ok(TestActsHelperOutput::default()) }));
    let goal_stack_helper = Arc::new(|_req| boxed(async { Ok(TestGoalStackPatch::default()) }));

    Arc::new(cortex_with_hooks(
        TestHooks::new(
            sense_helper,
            act_descriptor_helper,
            primary,
            acts_helper,
            goal_stack_helper,
        ),
        ReactionLimits::default(),
    ))
}

#[tokio::test]
async fn sleep_sense_breaks_loop_without_calling_cortex() {
    let (sense_tx, sense_rx) = mpsc::channel(2);
    sense_tx
        .send(Sense::Sleep)
        .await
        .expect("sleep should be enqueued");
    drop(sense_tx);

    let calls = Arc::new(AtomicUsize::new(0));
    let batch_sizes = Arc::new(Mutex::new(Vec::new()));
    let continuity = Arc::new(Mutex::new(ContinuityEngine::with_defaults()));
    let ledger = Arc::new(Mutex::new(LedgerStage::new(1_000)));

    let runtime = Stem::new(
        counting_cortex(Arc::clone(&calls), Arc::clone(&batch_sizes)),
        continuity,
        ledger,
        test_spine(),
        sense_rx,
    );
    runtime.run().await.expect("stem should exit cleanly");

    assert_eq!(calls.load(Ordering::Relaxed), 0);
    assert!(batch_sizes.lock().await.is_empty());
}

#[tokio::test]
async fn stem_drains_all_queued_senses_into_one_cortex_cycle() {
    let (sense_tx, sense_rx) = mpsc::channel(8);
    sense_tx
        .send(Sense::Domain(SenseDatum {
            sense_id: "sense:1".to_string(),
            endpoint_id: "ep.demo".to_string(),
            neural_signal_descriptor_id: "sense.demo".to_string(),
            payload: serde_json::json!({"a":1}),
        }))
        .await
        .expect("sense should enqueue");
    sense_tx
        .send(Sense::Domain(SenseDatum {
            sense_id: "sense:2".to_string(),
            endpoint_id: "ep.demo".to_string(),
            neural_signal_descriptor_id: "sense.demo".to_string(),
            payload: serde_json::json!({"b":2}),
        }))
        .await
        .expect("sense should enqueue");
    sense_tx
        .send(Sense::Sleep)
        .await
        .expect("sleep should enqueue");
    drop(sense_tx);

    let calls = Arc::new(AtomicUsize::new(0));
    let batch_sizes = Arc::new(Mutex::new(Vec::new()));
    let continuity = Arc::new(Mutex::new(ContinuityEngine::with_defaults()));
    let ledger = Arc::new(Mutex::new(LedgerStage::new(1_000)));

    let runtime = Stem::new(
        counting_cortex(Arc::clone(&calls), Arc::clone(&batch_sizes)),
        continuity,
        ledger,
        test_spine(),
        sense_rx,
    );
    runtime.run().await.expect("stem should exit cleanly");

    assert_eq!(calls.load(Ordering::Relaxed), 1);
    let recorded = batch_sizes.lock().await.clone();
    assert_eq!(recorded, vec![2]);
}
