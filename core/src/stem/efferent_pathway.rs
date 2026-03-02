use std::{
    collections::{BTreeMap, VecDeque},
    fmt,
    sync::Arc,
    time::Duration,
};

use tokio::{
    sync::{Mutex, mpsc},
    task::JoinHandle,
    time::{Instant, sleep_until},
};
use tokio_util::sync::CancellationToken;

use crate::{
    continuity::{ContinuityEngine, DispatchContext as ContinuityDispatchContext},
    spine::{ActDispatchResult, Spine},
    stem::runtime::StemControlPort,
    types::{Act, DispatchDecision, ProprioceptionDropPatch, ProprioceptionPatch},
};

const DEFAULT_EFFERENT_QUEUE_CAPACITY: usize = 128;
const DISPATCH_TERMINAL_RETENTION_LIMIT: usize = 128;

#[derive(Debug, Clone)]
pub struct EfferentActEnvelope {
    pub cycle_id: u64,
    pub act_seq_no: u64,
    pub act: Act,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EfferentEnqueueError {
    QueueClosed,
}

impl fmt::Display for EfferentEnqueueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::QueueClosed => write!(f, "efferent pathway queue is closed"),
        }
    }
}

impl std::error::Error for EfferentEnqueueError {}

#[derive(Clone)]
pub struct ActProducerHandle {
    tx: mpsc::Sender<EfferentActEnvelope>,
}

impl ActProducerHandle {
    pub async fn enqueue(
        &self,
        envelope: EfferentActEnvelope,
    ) -> Result<(), EfferentEnqueueError> {
        match self.tx.send(envelope).await {
            Ok(()) => Ok(()),
            Err(err) => {
                let dropped = err.0;
                tracing::warn!(
                    target = "stem.efferent",
                    cycle_id = dropped.cycle_id,
                    act_seq_no = dropped.act_seq_no,
                    act_instance_id = %dropped.act.act_instance_id,
                    "efferent_queue_closed"
                );
                Err(EfferentEnqueueError::QueueClosed)
            }
        }
    }
}

pub fn new_efferent_pathway(
    queue_capacity: Option<usize>,
) -> (ActProducerHandle, mpsc::Receiver<EfferentActEnvelope>) {
    let cap = queue_capacity
        .unwrap_or(DEFAULT_EFFERENT_QUEUE_CAPACITY)
        .max(1);
    let (tx, rx) = mpsc::channel(cap);
    (ActProducerHandle { tx }, rx)
}

pub fn spawn_efferent_runtime(
    mut rx: mpsc::Receiver<EfferentActEnvelope>,
    continuity: Arc<Mutex<ContinuityEngine>>,
    spine: Arc<Spine>,
    stem_control: Arc<dyn StemControlPort>,
    shutdown: CancellationToken,
    shutdown_drain_timeout: Duration,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut terminal_status_keys = VecDeque::new();
        let mut processed_count = 0_usize;
        let mut drain_deadline: Option<Instant> = None;

        loop {
            if let Some(deadline) = drain_deadline {
                tokio::select! {
                    maybe_task = rx.recv() => {
                        let Some(task) = maybe_task else {
                            tracing::info!(
                                target = "stem.efferent",
                                processed_count = processed_count,
                                "efferent_runtime_drain_complete"
                            );
                            break;
                        };
                        process_efferent_dispatch(
                            task,
                            &continuity,
                            &spine,
                            stem_control.as_ref(),
                            &mut terminal_status_keys,
                        ).await;
                        processed_count = processed_count.saturating_add(1);
                    }
                    _ = sleep_until(deadline) => {
                        let dropped_on_timeout = drain_remaining(&mut rx);
                        tracing::warn!(
                            target = "stem.efferent",
                            drain_timeout_ms = shutdown_drain_timeout.as_millis() as u64,
                            processed_count = processed_count,
                            dropped_on_timeout = dropped_on_timeout,
                            "efferent_runtime_drain_timeout_drop_remaining"
                        );
                        break;
                    }
                }
                continue;
            }

            tokio::select! {
                _ = shutdown.cancelled() => {
                    let timeout = shutdown_drain_timeout.max(Duration::from_millis(1));
                    drain_deadline = Some(Instant::now() + timeout);
                    tracing::info!(
                        target = "stem.efferent",
                        drain_timeout_ms = timeout.as_millis() as u64,
                        "efferent_runtime_shutdown_begin_drain"
                    );
                }
                maybe_task = rx.recv() => {
                    let Some(task) = maybe_task else {
                        tracing::info!(
                            target = "stem.efferent",
                            processed_count = processed_count,
                            "efferent_runtime_queue_closed"
                        );
                        break;
                    };
                    process_efferent_dispatch(
                        task,
                        &continuity,
                        &spine,
                        stem_control.as_ref(),
                        &mut terminal_status_keys,
                    ).await;
                    processed_count = processed_count.saturating_add(1);
                }
            }
        }
    })
}

async fn process_efferent_dispatch(
    task: EfferentActEnvelope,
    continuity: &Arc<Mutex<ContinuityEngine>>,
    spine: &Arc<Spine>,
    stem_control: &dyn StemControlPort,
    terminal_status_keys: &mut VecDeque<String>,
) {
    let status_key = dispatch_status_key(&task.act.act_instance_id);
    emit_status_patch(stem_control, &status_key, "DISPATCHING").await;

    let continuity_status = match continuity.lock().await.on_act(
        &task.act,
        &ContinuityDispatchContext {
            cycle_id: task.cycle_id,
            act_seq_no: task.act_seq_no,
        },
    ) {
        Ok(DispatchDecision::Continue) => None,
        Ok(DispatchDecision::Break) => Some("REJECTED"),
        Err(err) => {
            tracing::warn!(
                target: "stem.efferent",
                cycle_id = task.cycle_id,
                act_seq_no = task.act_seq_no,
                act_instance_id = %task.act.act_instance_id,
                error = %err,
                "continuity_dispatch_failed_mark_lost"
            );
            Some("LOST")
        }
    };

    let terminal_status = if let Some(status) = continuity_status {
        status
    } else {
        match spine.on_act_final(task.act.clone()).await {
            Ok(ActDispatchResult::Acknowledged { .. }) => "ACK",
            Ok(ActDispatchResult::Rejected { .. }) => "REJECTED",
            Ok(ActDispatchResult::Lost { .. }) => "LOST",
            Err(err) => {
                tracing::warn!(
                    target: "stem.efferent",
                    cycle_id = task.cycle_id,
                    act_seq_no = task.act_seq_no,
                    act_instance_id = %task.act.act_instance_id,
                    error = %err,
                    "spine_dispatch_failed_mark_lost"
                );
                "LOST"
            }
        }
    };

    emit_status_patch(stem_control, &status_key, terminal_status).await;

    terminal_status_keys.push_back(status_key.clone());
    if terminal_status_keys.len() > DISPATCH_TERMINAL_RETENTION_LIMIT
        && let Some(dropped_key) = terminal_status_keys.pop_front()
    {
        emit_status_drop(stem_control, dropped_key).await;
    }
}

fn dispatch_status_key(act_instance_id: &str) -> String {
    format!("stem.dispatch.{act_instance_id}.status")
}

async fn emit_status_patch(stem_control: &dyn StemControlPort, key: &str, value: &str) {
    let mut entries = BTreeMap::new();
    entries.insert(key.to_string(), value.to_string());
    stem_control
        .apply_proprioception_patch(ProprioceptionPatch { entries })
        .await;
}

async fn emit_status_drop(stem_control: &dyn StemControlPort, key: String) {
    stem_control
        .apply_proprioception_drop(ProprioceptionDropPatch { keys: vec![key] })
        .await;
}

fn drain_remaining(rx: &mut mpsc::Receiver<EfferentActEnvelope>) -> usize {
    let mut dropped = 0_usize;
    loop {
        match rx.try_recv() {
            Ok(_task) => {
                dropped = dropped.saturating_add(1);
            }
            Err(mpsc::error::TryRecvError::Empty) => break,
            Err(mpsc::error::TryRecvError::Disconnected) => break,
        }
    }
    dropped
}
