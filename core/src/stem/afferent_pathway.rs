use std::{
    fmt,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use async_trait::async_trait;
use serde_json::json;
use tokio::sync::{Mutex, mpsc, oneshot};

use crate::{
    observability::runtime as observability_runtime,
    stem::{ContinueOutput, PathwayMiddlewareDecision},
    types::Sense,
};

const DEFAULT_AFFERENT_WAIT_TIMEOUT_MS: u64 = 100;

pub type SenseIngressHandle = SenseAfferentPathway;
pub type AfferentControlHandle = SenseAfferentPathway;

pub type AfferentMiddlewareDecision = PathwayMiddlewareDecision<Sense, ()>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AfferentPathwayErrorKind {
    Closed,
    QueueClosed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AfferentPathwayError {
    pub kind: AfferentPathwayErrorKind,
    pub message: String,
}

impl AfferentPathwayError {
    fn closed() -> Self {
        Self {
            kind: AfferentPathwayErrorKind::Closed,
            message: "sense afferent pathway gate is closed".to_string(),
        }
    }

    fn queue_closed() -> Self {
        Self {
            kind: AfferentPathwayErrorKind::QueueClosed,
            message: "sense afferent pathway queue receiver is closed".to_string(),
        }
    }
}

impl fmt::Display for AfferentPathwayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AfferentPathwayError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AfferentDispatchResult {
    Accepted {
        reference_id: String,
    },
    Rejected {
        reason_code: String,
        reference_id: String,
    },
    Lost {
        reason_code: String,
        reference_id: String,
    },
}

#[async_trait]
pub trait AfferentMiddleware: Send + Sync {
    async fn handle_sense(
        &self,
        sense: &Sense,
    ) -> Result<AfferentMiddlewareDecision, AfferentPathwayError>;
}

pub struct SenseConsumerHandle {
    rx: mpsc::Receiver<Sense>,
}

impl SenseConsumerHandle {
    pub fn new(rx: mpsc::Receiver<Sense>) -> Self {
        Self { rx }
    }

    pub async fn recv(&mut self) -> Option<Sense> {
        self.rx.recv().await
    }

    pub fn try_recv(&mut self) -> Result<Sense, mpsc::error::TryRecvError> {
        self.rx.try_recv()
    }
}

#[derive(Debug)]
struct AfferentSenseEnvelope {
    seq_no: u64,
    sense: Sense,
    response_tx: Option<oneshot::Sender<AfferentDispatchResult>>,
}

enum AfferentBusCommand {
    Emit(AfferentSenseEnvelope),
}

#[derive(Clone)]
pub struct SenseAfferentPathway {
    gate_open: Arc<AtomicBool>,
    send_lock: Arc<Mutex<()>>,
    tx: mpsc::Sender<AfferentBusCommand>,
    next_seq_no: Arc<Mutex<u64>>,
}

impl SenseAfferentPathway {
    pub fn new(queue_capacity: usize, middleware: Vec<Arc<dyn AfferentMiddleware>>) -> Self {
        let (tx, rx) = mpsc::channel(queue_capacity.max(1));
        tokio::spawn(run_afferent_runtime(rx, middleware));
        Self {
            gate_open: Arc::new(AtomicBool::new(true)),
            send_lock: Arc::new(Mutex::new(())),
            tx,
            next_seq_no: Arc::new(Mutex::new(0)),
        }
    }

    pub fn new_handles(
        queue_capacity: usize,
        middleware: Vec<Arc<dyn AfferentMiddleware>>,
    ) -> (SenseIngressHandle, AfferentControlHandle) {
        let ingress = Self::new(queue_capacity, middleware);
        let control = ingress.clone();
        (ingress, control)
    }

    pub fn is_open(&self) -> bool {
        self.gate_open.load(Ordering::Acquire)
    }

    pub async fn emit_sense(&self, sense: Sense) -> Result<(), AfferentPathwayError> {
        self.enqueue(sense, None).await
    }

    pub async fn emit_sense_and_wait(&self, sense: Sense) -> AfferentDispatchResult {
        let reference_id = sense.sense_instance_id.clone();
        let (tx, rx) = oneshot::channel();
        if self.enqueue(sense, Some(tx)).await.is_err() {
            return AfferentDispatchResult::Lost {
                reason_code: "afferent_queue_closed".to_string(),
                reference_id,
            };
        }

        match tokio::time::timeout(
            std::time::Duration::from_millis(DEFAULT_AFFERENT_WAIT_TIMEOUT_MS),
            rx,
        )
        .await
        {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => AfferentDispatchResult::Lost {
                reason_code: "afferent_response_dropped".to_string(),
                reference_id,
            },
            Err(_) => AfferentDispatchResult::Lost {
                reason_code: "afferent_dispatch_timeout".to_string(),
                reference_id,
            },
        }
    }

    pub async fn send(&self, sense: Sense) -> Result<(), AfferentPathwayError> {
        self.emit_sense(sense).await
    }

    pub async fn close_gate(&self) {
        let _guard = self.send_lock.lock().await;
        self.gate_open.store(false, Ordering::Release);
    }

    async fn enqueue(
        &self,
        sense: Sense,
        response_tx: Option<oneshot::Sender<AfferentDispatchResult>>,
    ) -> Result<(), AfferentPathwayError> {
        let _guard = self.send_lock.lock().await;
        if !self.gate_open.load(Ordering::Acquire) {
            return Err(AfferentPathwayError::closed());
        }

        let seq_no = {
            let mut next = self.next_seq_no.lock().await;
            *next = next.saturating_add(1);
            *next
        };

        let endpoint_id = sense.endpoint_id.clone();
        let descriptor_id = sense.neural_signal_descriptor_id.clone();
        let sense_id = sense.sense_instance_id.clone();
        let sense_payload = json!(sense.payload.clone());
        let sense_weight = sense.weight;
        let envelope = AfferentSenseEnvelope {
            seq_no,
            sense,
            response_tx,
        };

        self.tx
            .send(AfferentBusCommand::Emit(envelope))
            .await
            .map_err(|_| AfferentPathwayError::queue_closed())?;

        observability_runtime::emit_stem_afferent(
            "enqueue",
            &descriptor_id,
            Some(&endpoint_id),
            Some(&sense_id),
            None,
            Some(sense_payload),
            Some(sense_weight),
            Some(json!({
                "queue_name": "afferent",
                "seq_no": seq_no,
            })),
            None,
            None,
        );
        Ok(())
    }
}

async fn run_afferent_runtime(
    mut rx: mpsc::Receiver<AfferentBusCommand>,
    middleware: Vec<Arc<dyn AfferentMiddleware>>,
) {
    while let Some(command) = rx.recv().await {
        match command {
            AfferentBusCommand::Emit(envelope) => {
                process_sense_envelope(envelope, &middleware).await;
            }
        }
    }
}

async fn process_sense_envelope(
    envelope: AfferentSenseEnvelope,
    middleware: &[Arc<dyn AfferentMiddleware>],
) {
    let AfferentSenseEnvelope {
        seq_no,
        sense,
        response_tx,
    } = envelope;
    let endpoint_id = sense.endpoint_id.clone();
    let descriptor_id = sense.neural_signal_descriptor_id.clone();
    let result = run_afferent_sequence(sense, middleware, 0).await;

    observability_runtime::emit_stem_afferent(
        "result",
        &descriptor_id,
        Some(&endpoint_id),
        Some(result_reference_id(&result)),
        None,
        None,
        None,
        Some(json!({
            "queue_name": "afferent",
            "seq_no": seq_no,
            "status": afferent_result_status(&result),
        })),
        None,
        result_reason_code(&result),
    );

    if let Some(tx) = response_tx {
        let _ = tx.send(result);
    }
}

async fn run_afferent_sequence(
    sense: Sense,
    middleware: &[Arc<dyn AfferentMiddleware>],
    start_index: usize,
) -> AfferentDispatchResult {
    if middleware.is_empty() || start_index >= middleware.len() {
        return AfferentDispatchResult::Rejected {
            reason_code: "route_not_found".to_string(),
            reference_id: sense.sense_instance_id,
        };
    }

    let mut current = vec![sense];
    for middleware_index in start_index..middleware.len() {
        let mut next = Vec::new();
        for signal in current {
            let reference_id = signal.sense_instance_id.clone();
            let decision = match middleware[middleware_index].handle_sense(&signal).await {
                Ok(decision) => decision,
                Err(_err) => {
                    return AfferentDispatchResult::Lost {
                        reason_code: format!("afferent_middleware_failed:{middleware_index}"),
                        reference_id,
                    };
                }
            };
            match decision {
                PathwayMiddlewareDecision::Accepted(()) => {
                    return AfferentDispatchResult::Accepted { reference_id };
                }
                PathwayMiddlewareDecision::Rejected { reason_code, .. } => {
                    return AfferentDispatchResult::Rejected {
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
        current = next;
        if current.is_empty() {
            return AfferentDispatchResult::Rejected {
                reason_code: "empty_transform".to_string(),
                reference_id: "afferent".to_string(),
            };
        }
    }

    let reference_id = current
        .into_iter()
        .next()
        .map(|sense| sense.sense_instance_id)
        .unwrap_or_else(|| "afferent".to_string());
    AfferentDispatchResult::Rejected {
        reason_code: "route_not_found".to_string(),
        reference_id,
    }
}

fn afferent_result_status(result: &AfferentDispatchResult) -> &'static str {
    match result {
        AfferentDispatchResult::Accepted { .. } => "ACCEPTED",
        AfferentDispatchResult::Rejected { .. } => "REJECTED",
        AfferentDispatchResult::Lost { .. } => "LOST",
    }
}

fn result_reference_id(result: &AfferentDispatchResult) -> &str {
    match result {
        AfferentDispatchResult::Accepted { reference_id }
        | AfferentDispatchResult::Rejected { reference_id, .. }
        | AfferentDispatchResult::Lost { reference_id, .. } => reference_id,
    }
}

fn result_reason_code(result: &AfferentDispatchResult) -> Option<&str> {
    match result {
        AfferentDispatchResult::Accepted { .. } => None,
        AfferentDispatchResult::Rejected { reason_code, .. }
        | AfferentDispatchResult::Lost { reason_code, .. } => Some(reason_code),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{Duration, timeout};

    struct RecordingAcceptMiddleware {
        tx: mpsc::Sender<String>,
    }

    #[async_trait]
    impl AfferentMiddleware for RecordingAcceptMiddleware {
        async fn handle_sense(
            &self,
            sense: &Sense,
        ) -> Result<AfferentMiddlewareDecision, AfferentPathwayError> {
            self.tx
                .send(sense.sense_instance_id.clone())
                .await
                .expect("recording channel should stay open");
            Ok(PathwayMiddlewareDecision::Accepted(()))
        }
    }

    #[tokio::test]
    async fn emit_sense_and_wait_returns_accepted_from_fixed_sequence() {
        let (record_tx, mut record_rx) = mpsc::channel(1);
        let pathway = SenseAfferentPathway::new(
            4,
            vec![Arc::new(RecordingAcceptMiddleware { tx: record_tx })],
        );
        let sense = Sense {
            sense_instance_id: "sense-1".to_string(),
            endpoint_id: "endpoint".to_string(),
            neural_signal_descriptor_id: "sense.test".to_string(),
            payload: "{}".to_string(),
            weight: 1.0,
            act_instance_id: None,
        };

        let result = pathway.emit_sense_and_wait(sense).await;

        assert_eq!(
            result,
            AfferentDispatchResult::Accepted {
                reference_id: "sense-1".to_string()
            }
        );
        assert_eq!(
            timeout(Duration::from_millis(100), record_rx.recv())
                .await
                .expect("recording should arrive"),
            Some("sense-1".to_string())
        );
    }
}
