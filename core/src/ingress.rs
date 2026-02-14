use std::{
    fmt,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use tokio::sync::{Mutex, mpsc};

use crate::runtime_types::Sense;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngressErrorKind {
    Closed,
    QueueClosed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngressError {
    pub kind: IngressErrorKind,
    pub message: String,
}

impl IngressError {
    fn closed() -> Self {
        Self {
            kind: IngressErrorKind::Closed,
            message: "sense ingress gate is closed".to_string(),
        }
    }

    fn queue_closed() -> Self {
        Self {
            kind: IngressErrorKind::QueueClosed,
            message: "sense queue receiver is closed".to_string(),
        }
    }
}

impl fmt::Display for IngressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for IngressError {}

#[derive(Clone)]
pub struct SenseIngress {
    gate_open: Arc<AtomicBool>,
    send_lock: Arc<Mutex<()>>,
    tx: mpsc::Sender<Sense>,
}

impl SenseIngress {
    pub fn new(tx: mpsc::Sender<Sense>) -> Self {
        Self {
            gate_open: Arc::new(AtomicBool::new(true)),
            send_lock: Arc::new(Mutex::new(())),
            tx,
        }
    }

    pub fn is_open(&self) -> bool {
        self.gate_open.load(Ordering::Acquire)
    }

    pub async fn send(&self, sense: Sense) -> Result<(), IngressError> {
        let _guard = self.send_lock.lock().await;
        if !self.gate_open.load(Ordering::Acquire) {
            return Err(IngressError::closed());
        }
        self.tx.send(sense).await.map_err(|_| IngressError::queue_closed())
    }

    pub async fn close_gate(&self) {
        let _guard = self.send_lock.lock().await;
        self.gate_open.store(false, Ordering::Release);
    }

    pub async fn send_sleep_blocking(&self) -> Result<(), IngressError> {
        let _guard = self.send_lock.lock().await;
        self.tx
            .send(Sense::Sleep)
            .await
            .map_err(|_| IngressError::queue_closed())
    }
}
