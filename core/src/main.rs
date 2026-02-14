use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::{
    signal::unix::{SignalKind, signal},
    sync::{Mutex, mpsc},
};
use tokio_util::sync::CancellationToken;

use beluna::{
    ai_gateway::{
        credentials::EnvCredentialProvider, gateway::AIGateway, telemetry::NoopTelemetrySink,
    },
    body::std::register_std_body_endpoints,
    cli::config_path_from_args,
    config::Config,
    continuity::ContinuityEngine,
    cortex::{
        AIGatewayAttemptExtractor, AIGatewayPayloadFiller, AIGatewayPrimaryReasoner,
        CortexPipeline, DeterministicAttemptClamp, NoopTelemetryPort,
    },
    ingress::SenseIngress,
    ledger::LedgerStage,
    spine::{
        EndpointRegistryPort, InMemoryEndpointRegistry, RoutingSpineExecutor, SpineExecutionMode,
        adapters::unix_socket::{BodyEndpointBroker, UnixSocketAdapter},
    },
    stem::{StemRuntime, register_default_native_endpoints},
};

const REMOTE_ENDPOINT_TIMEOUT_MS: u64 = 30_000;

#[tokio::main]
async fn main() -> Result<()> {
    let config_path = config_path_from_args()?;
    let config = Config::load(&config_path)
        .with_context(|| format!("failed to load config from {}", config_path.display()))?;

    let (sense_tx, sense_rx) = mpsc::channel(config.r#loop.sense_queue_capacity.max(1));
    let ingress = SenseIngress::new(sense_tx);

    let gateway = Arc::new(
        AIGateway::new(
            config.ai_gateway.clone(),
            Arc::new(EnvCredentialProvider),
            Arc::new(NoopTelemetrySink),
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
    let filler = Arc::new(AIGatewayPayloadFiller::new(
        Arc::clone(&gateway),
        config.cortex.sub_backend_id.clone(),
        None,
    ));
    let clamp = Arc::new(DeterministicAttemptClamp);
    let telemetry = Arc::new(NoopTelemetryPort);
    let cortex = Arc::new(CortexPipeline::new(
        primary,
        extractor,
        filler,
        clamp,
        telemetry,
        config.cortex.default_limits.clone(),
    ));

    let registry: Arc<dyn EndpointRegistryPort> = Arc::new(InMemoryEndpointRegistry::new());
    register_default_native_endpoints(Arc::clone(&registry))?;
    register_std_body_endpoints(
        Arc::clone(&registry),
        ingress.clone(),
        config.body.std_shell.enabled,
        config.body.std_shell.limits.clone(),
        config.body.std_web.enabled,
        config.body.std_web.limits.clone(),
    )?;

    let spine = Arc::new(RoutingSpineExecutor::new(
        SpineExecutionMode::SerializedDeterministic,
        Arc::clone(&registry),
    ));
    let continuity = Arc::new(Mutex::new(ContinuityEngine::with_defaults()));
    let ledger = Arc::new(Mutex::new(LedgerStage::new(1_000_000)));

    let stem_runtime = StemRuntime::new(cortex, continuity.clone(), ledger, spine, sense_rx);
    let stem_task = tokio::spawn(async move { stem_runtime.run().await });

    let shutdown = CancellationToken::new();
    let broker = Arc::new(BodyEndpointBroker::new(REMOTE_ENDPOINT_TIMEOUT_MS));
    let adapter = UnixSocketAdapter::new(config.socket_path.clone());
    let adapter_shutdown = shutdown.clone();
    let adapter_task = tokio::spawn({
        let ingress = ingress.clone();
        let registry = Arc::clone(&registry);
        let broker = Arc::clone(&broker);
        async move { adapter.run(ingress, registry, broker, adapter_shutdown).await }
    });

    eprintln!(
        "Beluna listening on unix socket (NDJSON): {}",
        config.socket_path.display()
    );

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
    match adapter_task.await {
        Ok(Ok(())) => {}
        Ok(Err(err)) => eprintln!("unix socket adapter exited with error: {err:#}"),
        Err(err) => eprintln!("unix socket adapter task join failed: {err}"),
    }

    eprintln!("Beluna stopped: received {signal_name}");
    Ok(())
}
