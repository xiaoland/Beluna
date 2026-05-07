use std::{future::Future, pin::Pin};

pub type MoiraTask = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

pub trait MoiraTaskSpawner: Send + Sync {
    fn spawn(&self, task: MoiraTask);
}

#[derive(Debug, Default)]
pub struct TokioTaskSpawner;

impl MoiraTaskSpawner for TokioTaskSpawner {
    fn spawn(&self, task: MoiraTask) {
        tokio::spawn(task);
    }
}
