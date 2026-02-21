use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::{
    signal::unix::{SignalKind, signal},
    sync::Mutex,
};
use tracing::Instrument;

use beluna::{
    afferent_pathway::SenseAfferentPathway,
    ai_gateway::{credentials::EnvCredentialProvider, gateway::AIGateway},
    body::register_inline_body_endpoints,
    cli::config_path_from_args,
    config::Config,
    continuity::ContinuityEngine,
    cortex::Cortex,
    ledger::LedgerStage,
    logging::init_tracing,
    observability::metrics::{MetricsRuntime, start_prometheus_exporter},
    spine::{Spine, global_spine, install_global_spine, shutdown_global_spine},
    stem::Stem,
};

#[tokio::main]
async fn main() -> Result<()> {
    let config_path = config_path_from_args()?;
    let config = Config::load(&config_path)
        .with_context(|| format!("failed to load config from {}", config_path.display()))?;
    let _logging_guard =
        init_tracing(&config.logging).context("failed to initialize tracing logging")?;
    let _metrics_runtime = match start_prometheus_exporter(MetricsRuntime::default_listen_addr()) {
        Ok(runtime) => {
            tracing::info!(
                target: "observability",
                listen_addr = %runtime.listen_addr,
                "prometheus_metrics_exporter_started"
            );
            Some(runtime)
        }
        Err(err) => {
            tracing::warn!(
                target: "observability",
                listen_addr = %MetricsRuntime::default_listen_addr(),
                error = %err,
                "prometheus_metrics_exporter_disabled"
            );
            None
        }
    };
    let _run_span = tracing::info_span!("core_run", run_id = %_logging_guard.run_id()).entered();
    tracing::info!(
        target: "core",
        config_path = %config_path.display(),
        "core_runtime_booting"
    );

    let (afferent_pathway, sense_rx) =
        SenseAfferentPathway::new(config.r#loop.sense_queue_capacity);

    let gateway = Arc::new(
        AIGateway::new(config.ai_gateway.clone(), Arc::new(EnvCredentialProvider))
            .context("failed to construct ai gateway for cortex")?,
    );

    let cortex = Arc::new(Cortex::from_config(
        &config.cortex,
        Arc::clone(&gateway),
        None,
    ));

    let spine_runtime = Spine::new(&config.spine, afferent_pathway.clone());
    install_global_spine(Arc::clone(&spine_runtime))
        .context("failed to initialize process-wide spine singleton")?;
    let inline_adapter = spine_runtime
        .inline_adapter()
        .context("inline body endpoints require spine.adapters entry with type=inline")?;
    register_inline_body_endpoints(
        inline_adapter,
        config.body.std_shell.enabled,
        config.body.std_shell.limits.clone(),
        config.body.std_web.enabled,
        config.body.std_web.limits.clone(),
    )?;

    let continuity = Arc::new(Mutex::new(ContinuityEngine::with_defaults()));
    let ledger = Arc::new(Mutex::new(LedgerStage::new(1_000_000)));

    let stem_runtime = Stem::new(cortex, continuity.clone(), ledger, spine_runtime, sense_rx);
    let stem_task = tokio::spawn(
        async move { stem_runtime.run().await }
            .instrument(tracing::info_span!(target: "core", "stem_task")),
    );

    let mut sigint =
        signal(SignalKind::interrupt()).context("unable to listen for SIGINT (Ctrl+C)")?;
    let mut sigterm = signal(SignalKind::terminate()).context("unable to listen for SIGTERM")?;

    let signal_name = tokio::select! {
        _ = sigint.recv() => "SIGINT",
        _ = sigterm.recv() => "SIGTERM",
    };

    tracing::info!(
        target: "core",
        signal_name = signal_name,
        "received_signal_closing_sense_afferent_pathway_gate"
    );
    afferent_pathway.close_gate().await;
    tracing::info!(target: "core", "enqueueing_sleep_sense");
    afferent_pathway
        .send_sleep_blocking()
        .await
        .context("failed to enqueue sleep sense")?;
    tracing::info!(target: "core", "sleep_sense_enqueued");

    stem_task.await.context("stem task join failed")??;
    continuity.lock().await.flush()?;

    let spine_runtime = global_spine().context("spine singleton is not initialized")?;
    shutdown_global_spine(spine_runtime).await?;

    tracing::info!(
        target: "core",
        signal_name = signal_name,
        "core_runtime_stopped"
    );
    Ok(())
}
