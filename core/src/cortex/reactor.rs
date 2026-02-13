use tokio::sync::mpsc;

use crate::cortex::{
    pipeline::CortexPipeline,
    types::{ReactionInput, ReactionResult},
};

pub struct CortexReactor {
    pipeline: CortexPipeline,
}

impl CortexReactor {
    pub fn new(pipeline: CortexPipeline) -> Self {
        Self { pipeline }
    }

    pub async fn react_once(&self, input: ReactionInput) -> ReactionResult {
        self.pipeline.react_once(input).await
    }

    pub async fn run(
        &self,
        mut inbox: mpsc::Receiver<ReactionInput>,
        outbox: mpsc::Sender<ReactionResult>,
    ) {
        while let Some(input) = inbox.recv().await {
            let result = self.react_once(input).await;
            if outbox.send(result).await.is_err() {
                break;
            }
        }
    }
}
