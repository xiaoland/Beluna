use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::{
    signal::unix::{SignalKind, signal},
    sync::{Mutex, mpsc},
};

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
    config::Config,
    continuity::ContinuityEngine,
    cortex::Cortex,
    ingress::SenseIngress,
    ledger::LedgerStage,
    spine::{Spine, global_spine, install_global_spine, shutdown_global_spine},
    stem::{Stem, register_default_native_endpoints},
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

    let cortex = Arc::new(Cortex::from_config(
        &config.cortex,
        Arc::clone(&gateway),
        None,
    ));

    let spine_runtime = Spine::new(&config.spine, ingress.clone());
    install_global_spine(Arc::clone(&spine_runtime))
        .context("failed to initialize process-wide spine singleton")?;
    let spine = global_spine().context("spine singleton is not initialized")?;
    register_default_native_endpoints(spine)?;
    register_std_body_endpoints(
        Arc::clone(&spine_runtime),
        ingress.clone(),
        config.body.std_shell.enabled,
        config.body.std_shell.limits.clone(),
        config.body.std_web.enabled,
        config.body.std_web.limits.clone(),
    )?;

    let continuity = Arc::new(Mutex::new(ContinuityEngine::with_defaults()));
    let ledger = Arc::new(Mutex::new(LedgerStage::new(1_000_000)));

    let stem_runtime = Stem::new(cortex, continuity.clone(), ledger, spine_runtime, sense_rx);
    let stem_task = tokio::spawn(async move { stem_runtime.run().await });

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

    let spine_runtime = global_spine().context("spine singleton is not initialized")?;
    shutdown_global_spine(spine_runtime).await?;

    eprintln!("Beluna stopped: received {signal_name}");
    Ok(())
}
