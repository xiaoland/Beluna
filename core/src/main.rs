use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::{
    signal::unix::{SignalKind, signal},
    sync::{Mutex, mpsc},
};
use tokio_util::sync::CancellationToken;

use beluna::{
    ai_gateway::{
        credentials::EnvCredentialProvider,
        gateway::AIGateway,
        telemetry::{
            NoopTelemetrySink, StderrTelemetrySink, TelemetrySink, ai_gateway_debug_enabled,
        },
    },
    body::std::register_std_body_endpoints,
    cli::config_path_from_args,
    config::{Config, SpineAdapterConfig},
    continuity::ContinuityEngine,
    cortex::{
        AIGatewayAttemptExtractor, AIGatewayPrimaryReasoner, CortexPipeline, NoopTelemetryPort,
    },
    ingress::SenseIngress,
    ledger::LedgerStage,
    spine::{
        EndpointRegistryPort, InMemoryEndpointRegistry, RoutingSpineExecutor, SpineExecutionMode,
        SpineExecutorPort, adapters::unix_socket::UnixSocketAdapter, global_executor,
        install_global_executor,
    },
    stem::{StemRuntime, register_default_native_endpoints},
};

#[tokio::main]
async fn main() -> Result<()> {
    let config_path = config_path_from_args()?;
    let config = Config::load(&config_path)
        .with_context(|| format!("failed to load config from {}", config_path.display()))?;

    let (sense_tx, sense_rx) = mpsc::channel(config.r#loop.sense_queue_capacity.max(1));
    let ingress = SenseIngress::new(sense_tx);

    let gateway_debug_enabled = ai_gateway_debug_enabled();
    let gateway_telemetry: Arc<dyn TelemetrySink> = if gateway_debug_enabled {
        Arc::new(StderrTelemetrySink)
    } else {
        Arc::new(NoopTelemetrySink)
    };
    if gateway_debug_enabled {
        eprintln!(
            "[ai_gateway] verbose debug logging enabled (set BELUNA_DEBUG_AI_GATEWAY=0/false to disable)"
        );
    }

    let gateway = Arc::new(
        AIGateway::new(
            config.ai_gateway.clone(),
            Arc::new(EnvCredentialProvider),
            gateway_telemetry,
        )
        .context("failed to construct ai gateway for cortex")?,
    );

    let primary = Arc::new(AIGatewayPrimaryReasoner::new(
        Arc::clone(&gateway),
        config.cortex.primary_backend_id.clone(),
        None,
    ));
    let extractor = Arc::new(AIGatewayAttemptExtractor::new(
        Arc::clone(&gateway),
        config.cortex.sub_backend_id.clone(),
        None,
    ));
    let telemetry = Arc::new(NoopTelemetryPort);
    let cortex = Arc::new(CortexPipeline::new(
        primary,
        extractor,
        telemetry,
        config.cortex.default_limits.clone(),
    ));

    let registry = Arc::new(InMemoryEndpointRegistry::new());
    let registry_port: Arc<dyn EndpointRegistryPort> = registry.clone();
    register_default_native_endpoints(Arc::clone(&registry_port))?;
    register_std_body_endpoints(
        Arc::clone(&registry_port),
        ingress.clone(),
        config.body.std_shell.enabled,
        config.body.std_shell.limits.clone(),
        config.body.std_web.enabled,
        config.body.std_web.limits.clone(),
    )?;

    let spine: Arc<dyn SpineExecutorPort> = Arc::new(RoutingSpineExecutor::new(
        SpineExecutionMode::SerializedDeterministic,
        Arc::clone(&registry_port),
    ));
    install_global_executor(Arc::clone(&spine))
        .context("failed to initialize process-wide spine singleton")?;
    let spine = global_executor().context("spine singleton is not initialized")?;

    let continuity = Arc::new(Mutex::new(ContinuityEngine::with_defaults()));
    let ledger = Arc::new(Mutex::new(LedgerStage::new(1_000_000)));

    let stem_runtime = StemRuntime::new(cortex, continuity.clone(), ledger, spine, sense_rx);
    let stem_task = tokio::spawn(async move { stem_runtime.run().await });

    let shutdown = CancellationToken::new();
    let mut adapter_tasks = Vec::new();

    for (index, adapter_config) in config.spine.adapters.iter().enumerate() {
        let adapter_id = (index as u64) + 1;
        match adapter_config {
            SpineAdapterConfig::UnixSocketNdjson {
                config: adapter_cfg,
            } => {
                let adapter = UnixSocketAdapter::new(adapter_cfg.socket_path.clone(), adapter_id);
                let adapter_shutdown = shutdown.clone();
                let ingress = ingress.clone();
                let registry = Arc::clone(&registry);
                let socket_path = adapter_cfg.socket_path.clone();
                let task =
                    tokio::spawn(
                        async move { adapter.run(ingress, registry, adapter_shutdown).await },
                    );
                adapter_tasks.push(task);
                eprintln!(
                    "Beluna listening on unix socket (NDJSON) adapter_id={}: {}",
                    adapter_id,
                    socket_path.display()
                );
            }
        }
    }

    let mut sigint =
        signal(SignalKind::interrupt()).context("unable to listen for SIGINT (Ctrl+C)")?;
    let mut sigterm = signal(SignalKind::terminate()).context("unable to listen for SIGTERM")?;

    let signal_name = tokio::select! {
        _ = sigint.recv() => "SIGINT",
        _ = sigterm.recv() => "SIGTERM",
    };

    eprintln!("received {signal_name}; closing sense ingress gate");
    ingress.close_gate().await;
    eprintln!("enqueueing sleep sense");
    ingress
        .send_sleep_blocking()
        .await
        .context("failed to enqueue sleep sense")?;
    eprintln!("sleep sense enqueued");

    stem_task.await.context("stem task join failed")??;
    continuity.lock().await.flush()?;

    shutdown.cancel();
    for adapter_task in adapter_tasks {
        match adapter_task.await {
            Ok(Ok(())) => {}
            Ok(Err(err)) => eprintln!("spine adapter exited with error: {err:#}"),
            Err(err) => eprintln!("spine adapter task join failed: {err}"),
        }
    }

    eprintln!("Beluna stopped: received {signal_name}");
    Ok(())
}
