use std::{fmt, sync::Arc, time::Duration};

use async_trait::async_trait;
use serde_json::json;
use tokio::{
    sync::{Mutex, mpsc, oneshot},
    task::JoinHandle,
    time::{Instant, sleep_until, timeout},
};
use tokio_util::sync::CancellationToken;

use crate::{
    continuity::{ContinuityEngine, DispatchContext as ContinuityDispatchContext},
    observability::runtime::{self as observability_runtime, DispatchOutcomeClass},
    spine::{ActDispatchResult, Spine},
    stem::{ContinueOutput, PathwayMiddlewareDecision},
    types::{Act, DispatchDecision},
};

const DEFAULT_EFFERENT_QUEUE_CAPACITY: usize = 128;

#[derive(Debug)]
pub struct EfferentActEnvelope {
    pub cycle_id: u64,
    pub act_seq_no: u64,
    pub act: Act,
    pub response_tx: Option<oneshot::Sender<ActDispatchResult>>,
}

impl EfferentActEnvelope {
    pub fn new(cycle_id: u64, act_seq_no: u64, act: Act) -> Self {
        Self {
            cycle_id,
            act_seq_no,
            act,
            response_tx: None,
        }
    }

    pub fn with_response(cycle_id: u64, act_seq_no: u64, act: Act) -> Self {
        Self::new(cycle_id, act_seq_no, act)
    }
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

pub type EfferentMiddlewareDecision = PathwayMiddlewareDecision<Act, ActDispatchResult>;

#[derive(Debug, Clone, Copy)]
pub struct EfferentMiddlewareContext {
    pub cycle_id: u64,
    pub act_seq_no: u64,
}

#[async_trait]
pub trait EfferentMiddleware: Send + Sync {
    async fn handle_act(
        &self,
        ctx: &EfferentMiddlewareContext,
        act: &Act,
    ) -> EfferentMiddlewareDecision;
}

pub struct ContinuityEfferentMiddleware {
    continuity: Arc<Mutex<ContinuityEngine>>,
}

impl ContinuityEfferentMiddleware {
    pub fn new(continuity: Arc<Mutex<ContinuityEngine>>) -> Self {
        Self { continuity }
    }
}

#[async_trait]
impl EfferentMiddleware for ContinuityEfferentMiddleware {
    async fn handle_act(
        &self,
        ctx: &EfferentMiddlewareContext,
        act: &Act,
    ) -> EfferentMiddlewareDecision {
        match self.continuity.lock().await.on_act(
            act,
            &ContinuityDispatchContext {
                cycle_id: ctx.cycle_id,
                act_seq_no: ctx.act_seq_no,
            },
        ) {
            Ok(DispatchDecision::Continue) => {
                PathwayMiddlewareDecision::Continue(ContinueOutput::Original)
            }
            Ok(DispatchDecision::Break) => PathwayMiddlewareDecision::Rejected {
                reason_code: "continuity_break".to_string(),
                message: None,
            },
            Err(err) => PathwayMiddlewareDecision::Accepted(ActDispatchResult::Lost {
                reason_code: format!("continuity_dispatch_failed:{err}"),
                reference_id: act.act_instance_id.clone(),
            }),
        }
    }
}

pub struct SpineEfferentMiddleware {
    spine: Arc<Spine>,
}

impl SpineEfferentMiddleware {
    pub fn new(spine: Arc<Spine>) -> Self {
        Self { spine }
    }
}

#[async_trait]
impl EfferentMiddleware for SpineEfferentMiddleware {
    async fn handle_act(
        &self,
        ctx: &EfferentMiddlewareContext,
        act: &Act,
    ) -> EfferentMiddlewareDecision {
        let result = match self.spine.on_act_final(ctx.cycle_id, act.clone()).await {
            Ok(result) => result,
            Err(err) => {
                tracing::warn!(
                    target = "stem.efferent",
                    cycle_id = ctx.cycle_id,
                    act_seq_no = ctx.act_seq_no,
                    act_instance_id = %act.act_instance_id,
                    error = %err,
                    "spine_dispatch_failed_mark_lost"
                );
                ActDispatchResult::Lost {
                    reason_code: "spine_dispatch_failed".to_string(),
                    reference_id: act.act_instance_id.clone(),
                }
            }
        };
        PathwayMiddlewareDecision::Accepted(result)
    }
}

pub type ActProducerHandle = EfferentTx;

#[derive(Clone)]
pub struct EfferentTx {
    tx: mpsc::Sender<EfferentActEnvelope>,
}

impl EfferentTx {
    pub async fn emit_act(
        &self,
        cycle_id: u64,
        act_seq_no: u64,
        act: Act,
    ) -> Result<(), EfferentEnqueueError> {
        self.enqueue(EfferentActEnvelope::new(cycle_id, act_seq_no, act))
            .await
    }

    pub async fn emit_act_and_wait(
        &self,
        cycle_id: u64,
        act_seq_no: u64,
        act: Act,
        wait_timeout: Duration,
    ) -> ActDispatchResult {
        self.dispatch_and_wait(
            EfferentActEnvelope::with_response(cycle_id, act_seq_no, act),
            wait_timeout,
        )
        .await
    }

    pub async fn enqueue(&self, envelope: EfferentActEnvelope) -> Result<(), EfferentEnqueueError> {
        let cycle_id = envelope.cycle_id;
        let act_id = envelope.act.act_instance_id.clone();
        let descriptor_id = envelope.act.neural_signal_descriptor_id.clone();
        let endpoint_id = envelope.act.endpoint_id.clone();
        let act_payload = envelope.act.payload.clone();
        match self.tx.send(envelope).await {
            Ok(()) => {
                observability_runtime::emit_stem_efferent(
                    "enqueue",
                    &act_id,
                    Some(descriptor_id.as_str()),
                    Some(endpoint_id.as_str()),
                    Some(act_payload),
                    Some(json!({ "queue_name": "efferent" })),
                    Some(cycle_id),
                    None,
                    None,
                    None,
                );
                Ok(())
            }
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

    pub async fn dispatch_and_wait(
        &self,
        mut envelope: EfferentActEnvelope,
        wait_timeout: Duration,
    ) -> ActDispatchResult {
        let (tx, rx) = oneshot::channel();
        let reference_id = envelope.act.act_instance_id.clone();
        envelope.response_tx = Some(tx);

        if self.enqueue(envelope).await.is_err() {
            return ActDispatchResult::Lost {
                reason_code: "efferent_queue_closed".to_string(),
                reference_id,
            };
        }

        match timeout(wait_timeout.max(Duration::from_millis(1)), rx).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => ActDispatchResult::Lost {
                reason_code: "efferent_response_dropped".to_string(),
                reference_id,
            },
            Err(_) => ActDispatchResult::Lost {
                reason_code: "efferent_dispatch_timeout".to_string(),
                reference_id,
            },
        }
    }
}

pub fn new_efferent_pathway(
    queue_capacity: Option<usize>,
) -> (EfferentTx, mpsc::Receiver<EfferentActEnvelope>) {
    let cap = queue_capacity
        .unwrap_or(DEFAULT_EFFERENT_QUEUE_CAPACITY)
        .max(1);
    let (tx, rx) = mpsc::channel(cap);
    (EfferentTx { tx }, rx)
}

pub fn spawn_efferent_runtime(
    mut rx: mpsc::Receiver<EfferentActEnvelope>,
    middleware: Vec<Arc<dyn EfferentMiddleware>>,
    shutdown: CancellationToken,
    shutdown_drain_timeout: Duration,
) -> JoinHandle<()> {
    tokio::spawn(async move {
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
                            &middleware,
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
                        &middleware,
                    ).await;
                    processed_count = processed_count.saturating_add(1);
                }
            }
        }
    })
}

async fn process_efferent_dispatch(
    task: EfferentActEnvelope,
    middleware: &[Arc<dyn EfferentMiddleware>],
) {
    let EfferentActEnvelope {
        cycle_id,
        act_seq_no,
        act,
        response_tx,
    } = task;

    observability_runtime::emit_stem_efferent(
        "dispatch",
        &act.act_instance_id,
        Some(&act.neural_signal_descriptor_id),
        Some(&act.endpoint_id),
        Some(act.payload.clone()),
        Some(json!({
            "queue_name": "efferent",
            "status": "dispatching",
        })),
        Some(cycle_id),
        None,
        None,
        None,
    );

    let dispatch_result = run_efferent_sequence(
        act.clone(),
        middleware,
        &EfferentMiddlewareContext {
            cycle_id,
            act_seq_no,
        },
    )
    .await;

    let terminal_status = dispatch_terminal_status(&dispatch_result);
    observability_runtime::emit_stem_efferent(
        "result",
        &act.act_instance_id,
        Some(&act.neural_signal_descriptor_id),
        Some(&act.endpoint_id),
        Some(act.payload.clone()),
        Some(json!({
            "queue_name": "efferent",
            "status": terminal_status,
        })),
        Some(cycle_id),
        None,
        Some(dispatch_outcome_class(&dispatch_result)),
        Some(dispatch_result_reference(&dispatch_result)),
    );

    if let Some(tx) = response_tx {
        let _ = tx.send(dispatch_result);
    }
}

async fn run_efferent_sequence(
    act: Act,
    middleware: &[Arc<dyn EfferentMiddleware>],
    ctx: &EfferentMiddlewareContext,
) -> ActDispatchResult {
    if middleware.is_empty() {
        return ActDispatchResult::Rejected {
            reason_code: "route_not_found".to_string(),
            reference_id: act.act_instance_id,
        };
    }

    let mut current = vec![act];
    for stage in middleware {
        let mut next = Vec::new();
        for signal in current {
            let reference_id = signal.act_instance_id.clone();
            match stage.handle_act(ctx, &signal).await {
                PathwayMiddlewareDecision::Accepted(result) => return result,
                PathwayMiddlewareDecision::Rejected { reason_code, .. } => {
                    return ActDispatchResult::Rejected {
                        reason_code,
                        reference_id,
                    };
                }
                PathwayMiddlewareDecision::Continue(ContinueOutput::Original) => {
                    next.push(signal);
                }
                PathwayMiddlewareDecision::Continue(ContinueOutput::Replace(signals)) => {
                    next.extend(signals);
                }
            }
        }
        if next.is_empty() {
            return ActDispatchResult::Rejected {
                reason_code: "empty_transform".to_string(),
                reference_id: "efferent".to_string(),
            };
        }
        current = next;
    }

    let reference_id = current
        .into_iter()
        .next()
        .map(|act| act.act_instance_id)
        .unwrap_or_else(|| "efferent".to_string());
    ActDispatchResult::Rejected {
        reason_code: "route_not_found".to_string(),
        reference_id,
    }
}

fn dispatch_terminal_status(dispatch_result: &ActDispatchResult) -> &'static str {
    match dispatch_result {
        ActDispatchResult::Acknowledged { .. } => "ACK",
        ActDispatchResult::Rejected { .. } => "REJECTED",
        ActDispatchResult::Lost { .. } => "LOST",
    }
}

fn dispatch_outcome_class(dispatch_result: &ActDispatchResult) -> DispatchOutcomeClass {
    match dispatch_result {
        ActDispatchResult::Acknowledged { .. } => DispatchOutcomeClass::Acknowledged,
        ActDispatchResult::Rejected { .. } => DispatchOutcomeClass::Rejected,
        ActDispatchResult::Lost { .. } => DispatchOutcomeClass::Lost,
    }
}

fn dispatch_result_reference(dispatch_result: &ActDispatchResult) -> serde_json::Value {
    match dispatch_result {
        ActDispatchResult::Acknowledged { reference_id } => json!({
            "reference_id": reference_id,
        }),
        ActDispatchResult::Rejected {
            reason_code,
            reference_id,
        }
        | ActDispatchResult::Lost {
            reason_code,
            reference_id,
        } => json!({
            "reason_code": reason_code,
            "reference_id": reference_id,
        }),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    struct PassThroughMiddleware;

    #[async_trait]
    impl EfferentMiddleware for PassThroughMiddleware {
        async fn handle_act(
            &self,
            _ctx: &EfferentMiddlewareContext,
            _act: &Act,
        ) -> EfferentMiddlewareDecision {
            PathwayMiddlewareDecision::Continue(ContinueOutput::Original)
        }
    }

    struct AcknowledgeMiddleware;

    #[async_trait]
    impl EfferentMiddleware for AcknowledgeMiddleware {
        async fn handle_act(
            &self,
            _ctx: &EfferentMiddlewareContext,
            act: &Act,
        ) -> EfferentMiddlewareDecision {
            PathwayMiddlewareDecision::Accepted(ActDispatchResult::Acknowledged {
                reference_id: act.act_instance_id.clone(),
            })
        }
    }

    #[tokio::test]
    async fn emit_act_and_wait_uses_fixed_middleware_sequence() {
        let (tx, rx) = new_efferent_pathway(Some(4));
        let shutdown = CancellationToken::new();
        let task = spawn_efferent_runtime(
            rx,
            vec![
                Arc::new(PassThroughMiddleware),
                Arc::new(AcknowledgeMiddleware),
            ],
            shutdown.clone(),
            Duration::from_millis(10),
        );
        let act = Act {
            act_instance_id: "act-1".to_string(),
            endpoint_id: "endpoint".to_string(),
            neural_signal_descriptor_id: "act.test".to_string(),
            might_emit_sense_ids: Vec::new(),
            payload: json!({}),
        };

        let result = tx
            .emit_act_and_wait(1, 1, act, Duration::from_millis(100))
            .await;

        assert_eq!(
            result,
            ActDispatchResult::Acknowledged {
                reference_id: "act-1".to_string()
            }
        );
        drop(tx);
        shutdown.cancel();
        task.await.expect("efferent runtime should join");
    }
}
