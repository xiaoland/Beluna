use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use async_trait::async_trait;
use tokio::sync::{Mutex, mpsc};

use beluna::{
    continuity::ContinuityEngine,
    cortex::{CortexError, CortexOutput, CortexPort},
    ledger::LedgerStage,
    runtime_types::{CognitionState, PhysicalState, Sense, SenseDatum},
    spine::{DeterministicNoopSpine, SpineExecutorPort},
    stem::StemRuntime,
};

struct CountingCortex {
    calls: Arc<AtomicUsize>,
    batch_sizes: Arc<Mutex<Vec<usize>>>,
}

#[async_trait]
impl CortexPort for CountingCortex {
    async fn cortex(
        &self,
        senses: &[Sense],
        _physical_state: &PhysicalState,
        cognition_state: &CognitionState,
    ) -> Result<CortexOutput, CortexError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        self.batch_sizes.lock().await.push(senses.len());
        Ok(CortexOutput {
            acts: Vec::new(),
            new_cognition_state: cognition_state.clone(),
        })
    }
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
    let cortex = Arc::new(CountingCortex {
        calls: Arc::clone(&calls),
        batch_sizes,
    });
    let continuity = Arc::new(Mutex::new(ContinuityEngine::with_defaults()));
    let ledger = Arc::new(Mutex::new(LedgerStage::new(1_000)));
    let spine: Arc<dyn SpineExecutorPort> = Arc::new(DeterministicNoopSpine::default());

    let runtime = StemRuntime::new(cortex, continuity, ledger, spine, sense_rx);
    runtime.run().await.expect("stem should exit cleanly");

    assert_eq!(calls.load(Ordering::Relaxed), 0);
}

#[tokio::test]
async fn stem_drains_all_queued_senses_into_one_cortex_cycle() {
    let (sense_tx, sense_rx) = mpsc::channel(8);
    sense_tx
        .send(Sense::Domain(SenseDatum {
            sense_id: "41f25f33-99f5-4250-99c3-020f8a92e199".to_string(),
            source: "test".to_string(),
            payload: serde_json::json!({"a":1}),
        }))
        .await
        .expect("sense should enqueue");
    sense_tx
        .send(Sense::Domain(SenseDatum {
            sense_id: "sense:2".to_string(),
            source: "test".to_string(),
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
    let cortex = Arc::new(CountingCortex {
        calls: Arc::clone(&calls),
        batch_sizes: Arc::clone(&batch_sizes),
    });
    let continuity = Arc::new(Mutex::new(ContinuityEngine::with_defaults()));
    let ledger = Arc::new(Mutex::new(LedgerStage::new(1_000)));
    let spine: Arc<dyn SpineExecutorPort> = Arc::new(DeterministicNoopSpine::default());

    let runtime = StemRuntime::new(cortex, continuity, ledger, spine, sense_rx);
    runtime.run().await.expect("stem should exit cleanly");

    assert_eq!(calls.load(Ordering::Relaxed), 1);
    let recorded = batch_sizes.lock().await.clone();
    assert_eq!(recorded, vec![2]);
}
