use std::{
    fmt,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use tokio::sync::{Mutex, mpsc};

use crate::types::Sense;

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

#[derive(Clone)]
pub struct SenseAfferentPathway {
    gate_open: Arc<AtomicBool>,
    send_lock: Arc<Mutex<()>>,
    tx: mpsc::Sender<Sense>,
}

impl SenseAfferentPathway {
    pub fn new(queue_capacity: usize) -> (Self, mpsc::Receiver<Sense>) {
        let (tx, rx) = mpsc::channel(queue_capacity.max(1));
        (Self::from_sender(tx), rx)
    }

    pub fn from_sender(tx: mpsc::Sender<Sense>) -> Self {
        Self {
            gate_open: Arc::new(AtomicBool::new(true)),
            send_lock: Arc::new(Mutex::new(())),
            tx,
        }
    }

    pub fn is_open(&self) -> bool {
        self.gate_open.load(Ordering::Acquire)
    }

    pub async fn send(&self, sense: Sense) -> Result<(), AfferentPathwayError> {
        let _guard = self.send_lock.lock().await;
        if !self.gate_open.load(Ordering::Acquire) {
            return Err(AfferentPathwayError::closed());
        }
        self.tx
            .send(sense)
            .await
            .map_err(|_| AfferentPathwayError::queue_closed())
    }

    pub async fn close_gate(&self) {
        let _guard = self.send_lock.lock().await;
        self.gate_open.store(false, Ordering::Release);
    }

    pub async fn send_hibernate_blocking(&self) -> Result<(), AfferentPathwayError> {
        let _guard = self.send_lock.lock().await;
        self.tx
            .send(Sense::Hibernate)
            .await
            .map_err(|_| AfferentPathwayError::queue_closed())
    }
}
