use std::sync::Arc;

use anyhow::Result;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::{
    config::SpineRuntimeConfig,
    ingress::SenseIngress,
    spine::{adapters::unix_socket::UnixSocketAdapter, registry::InMemoryEndpointRegistry},
};

pub struct SpineAdapterRuntime {
    tasks: Vec<JoinHandle<Result<()>>>,
}

impl SpineAdapterRuntime {
    pub fn start(
        config: &SpineRuntimeConfig,
        ingress: SenseIngress,
        registry: Arc<InMemoryEndpointRegistry>,
        shutdown: CancellationToken,
    ) -> Self {
        let mut tasks = Vec::new();

        for (index, adapter_config) in config.adapters.iter().enumerate() {
            let adapter_id = (index as u64) + 1;
            match adapter_config {
                crate::config::SpineAdapterConfig::UnixSocketNdjson {
                    config: adapter_cfg,
                } => {
                    let adapter =
                        UnixSocketAdapter::new(adapter_cfg.socket_path.clone(), adapter_id);
                    let ingress = ingress.clone();
                    let registry = Arc::clone(&registry);
                    let shutdown = shutdown.clone();
                    let socket_path = adapter_cfg.socket_path.clone();

                    let task = tokio::spawn(async move {
                        eprintln!(
                            "[spine] adapter_started type=unix-socket-ndjson adapter_id={} socket_path={}",
                            adapter_id,
                            socket_path.display()
                        );
                        adapter.run(ingress, registry, shutdown).await
                    });
                    tasks.push(task);
                }
            }
        }

        Self { tasks }
    }

    pub async fn join_all(self) {
        for task in self.tasks {
            match task.await {
                Ok(Ok(())) => {}
                Ok(Err(err)) => eprintln!("spine adapter exited with error: {err:#}"),
                Err(err) => eprintln!("spine adapter task join failed: {err}"),
            }
        }
    }
}
