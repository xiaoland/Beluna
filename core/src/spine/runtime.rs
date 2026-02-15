use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::{
    config::SpineRuntimeConfig,
    ingress::SenseIngress,
    spine::{
        EndpointRegistryPort, RoutingSpineExecutor, SpineExecutionMode, SpineExecutorPort,
        adapters::unix_socket::UnixSocketAdapter, registry::InMemoryEndpointRegistry,
    },
};

pub struct Spine {
    registry: Arc<InMemoryEndpointRegistry>,
    executor: Arc<dyn SpineExecutorPort>,
    shutdown: CancellationToken,
    tasks: Vec<JoinHandle<Result<()>>>,
}

impl Spine {
    pub fn new(config: &SpineRuntimeConfig, ingress: SenseIngress) -> Self {
        let registry = Arc::new(InMemoryEndpointRegistry::new());
        let registry_port: Arc<dyn EndpointRegistryPort> = registry.clone();
        let executor: Arc<dyn SpineExecutorPort> = Arc::new(RoutingSpineExecutor::new(
            SpineExecutionMode::SerializedDeterministic,
            Arc::clone(&registry_port),
        ));

        let shutdown = CancellationToken::new();
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

        Self {
            registry,
            executor,
            shutdown,
            tasks,
        }
    }

    pub fn registry_port(&self) -> Arc<dyn EndpointRegistryPort> {
        self.registry.clone()
    }

    pub fn executor_port(&self) -> Arc<dyn SpineExecutorPort> {
        Arc::clone(&self.executor)
    }

    pub async fn shutdown(self) {
        self.shutdown.cancel();
        for task in self.tasks {
            match task.await {
                Ok(Ok(())) => {}
                Ok(Err(err)) => eprintln!("spine adapter exited with error: {err:#}"),
                Err(err) => eprintln!("spine adapter task join failed: {err}"),
            }
        }
    }
}

pub struct SpineHandle {
    inner: Arc<Spine>,
}

impl SpineHandle {
    pub fn new(inner: Arc<Spine>) -> Self {
        Self { inner }
    }

    pub fn registry_port(&self) -> Arc<dyn EndpointRegistryPort> {
        self.inner.registry_port()
    }

    pub fn executor_port(&self) -> Arc<dyn SpineExecutorPort> {
        self.inner.executor_port()
    }
}

pub async fn shutdown_global_spine(spine: Arc<Spine>) -> Result<()> {
    match Arc::try_unwrap(spine) {
        Ok(spine) => {
            spine.shutdown().await;
            Ok(())
        }
        Err(_) => Err(anyhow::anyhow!(
            "failed to shutdown spine: outstanding references still exist"
        ))
        .context("spine shutdown requires exclusive ownership"),
    }
}
