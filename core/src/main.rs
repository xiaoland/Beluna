use std::{collections::BTreeMap, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use async_trait::async_trait;
use sysinfo::{Networks, System};
use tokio::{
    signal::unix::{SignalKind, signal},
    sync::{Mutex, RwLock, mpsc},
};
use tokio_util::sync::CancellationToken;
use tracing::Instrument;

use beluna::{
    ai_gateway::{chat::Chat, credentials::EnvCredentialProvider},
    body::start_inline_body_endpoints,
    cli::config_path_from_args,
    config::{Config, TickMissedBehavior},
    continuity::ContinuityEngine,
    cortex::{Cortex, CortexDeps, CortexRuntime, PhysicalStateReadPort},
    logging::init_tracing,
    observability::otel::OpenTelemetryRuntime,
    spine::{Spine, shutdown_global_spine},
    stem::{
        AfferentControlHandle, AfferentRuleControlPort, SenseAfferentPathway, StemControlPort,
        StemDeps, StemPhysicalStateStore, StemTickRuntime, new_efferent_pathway,
        spawn_efferent_runtime,
    },
    types::PhysicalState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppState {
    Init,
    Starting,
    Running,
    Closing,
    Closed,
}

#[derive(Clone)]
struct AppLifecycle {
    state: Arc<RwLock<AppState>>,
}

impl AppLifecycle {
    fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(AppState::Init)),
        }
    }

    async fn set(&self, next: AppState) {
        let mut guard = self.state.write().await;
        *guard = next;
    }
}

#[derive(Clone)]
struct AppContext {
    lifecycle: AppLifecycle,
    shutdown: CancellationToken,
    afferent_control: AfferentControlHandle,
    continuity: Arc<Mutex<ContinuityEngine>>,
    spine: Arc<Spine>,
}

struct MainPhysicalStateReader {
    stem_state: Arc<StemPhysicalStateStore>,
}

#[async_trait]
impl PhysicalStateReadPort for MainPhysicalStateReader {
    async fn snapshot(&self, cycle_id: u64) -> Result<PhysicalState> {
        Ok(self.stem_state.snapshot_for_cycle(cycle_id).await)
    }
}

fn collect_main_startup_proprioception() -> BTreeMap<String, String> {
    let mut entries = BTreeMap::new();
    entries.insert("main.os".to_string(), collect_os_summary());
    entries.insert("main.resources".to_string(), collect_resource_summary());
    entries.insert("main.network".to_string(), collect_network_summary());
    entries.insert(
        "main.cwd".to_string(),
        std::env::current_dir()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|err| format!("cwd_unavailable:{err}")),
    );
    entries.insert("main.pid".to_string(), std::process::id().to_string());
    entries
}

fn collect_os_summary() -> String {
    let name = System::name().unwrap_or_else(|| "unknown".to_string());
    let kernel = System::kernel_version().unwrap_or_else(|| "unknown".to_string());
    let os_version = System::os_version().unwrap_or_else(|| "unknown".to_string());
    let long_os_version = System::long_os_version().unwrap_or_else(|| "unknown".to_string());
    let host = System::host_name().unwrap_or_else(|| "unknown".to_string());
    format!(
        "name={name};kernel={kernel};os_version={os_version};long_os_version={long_os_version};host={host}"
    )
}

fn collect_resource_summary() -> String {
    let system = System::new_all();
    let total_memory = system.total_memory();
    let available_memory = system.available_memory();
    let used_memory = system.used_memory();
    let total_swap = system.total_swap();
    let used_swap = system.used_swap();
    let cpu_count = system.cpus().len();
    let uptime_seconds = System::uptime();
    format!(
        "cpu_count={cpu_count};memory(total={total_memory},available={available_memory},used={used_memory});swap(total={total_swap},used={used_swap});uptime_seconds={uptime_seconds}"
    )
}

fn collect_network_summary() -> String {
    let networks = Networks::new_with_refreshed_list();
    let mut names = networks
        .keys()
        .map(|name| name.to_string())
        .collect::<Vec<_>>();
    names.sort();
    format!(
        "interface_count={};interfaces={}",
        names.len(),
        names.join(",")
    )
}

#[tokio::main]
async fn main() -> Result<()> {
    let config_path = config_path_from_args()?;
    let config = Config::load(&config_path)
        .with_context(|| format!("failed to load config from {}", config_path.display()))?;
    let observability_runtime = OpenTelemetryRuntime::init(&config.observability)
        .context("failed to initialize OpenTelemetry runtime")?;
    let _logging_guard = init_tracing(
        &config.logging,
        observability_runtime.log_layer(),
        observability_runtime.trace_layer(),
    )
    .context("failed to initialize tracing logging")?;
    for state in observability_runtime.signal_states() {
        tracing::info!(
            target: "observability",
            signal = state.signal,
            requested = state.requested,
            enabled = state.enabled,
            protocol = ?state.protocol,
            endpoint = ?state.endpoint,
            timeout_ms = ?state.timeout_ms,
            detail = ?state.detail,
            "opentelemetry_otlp_signal_state"
        );
    }
    let _run_span = tracing::info_span!("core_run", run_id = %_logging_guard.run_id()).entered();
    tracing::info!(
        target: "core",
        config_path = %config_path.display(),
        "core_runtime_booting"
    );

    let lifecycle = AppLifecycle::new();
    lifecycle.set(AppState::Starting).await;

    if !matches!(config.r#loop.tick_missed_behavior, TickMissedBehavior::Skip) {
        tracing::warn!(target: "core", "unsupported_tick_missed_behavior_fallback_to_skip");
    }

    let (afferent_ingress, afferent_consumer, afferent_control) = SenseAfferentPathway::new_handles(
        config.r#loop.sense_queue_capacity,
        config.r#loop.max_deferring_nums,
        config.r#loop.afferent_sidecar_capacity,
    );
    let stem_state = Arc::new(StemPhysicalStateStore::new(
        collect_main_startup_proprioception(),
    ));
    let stem_control: Arc<dyn StemControlPort> = stem_state.clone();

    let chat = Arc::new(
        Chat::new(&config.ai_gateway, Arc::new(EnvCredentialProvider))
            .context("failed to construct chat runtime for cortex")?,
    );

    let spine_runtime = Spine::new(
        &config.spine,
        afferent_ingress.clone(),
        stem_control.clone(),
    );
    let inline_adapter = spine_runtime
        .inline_adapter()
        .context("inline body endpoints require spine.adapters entry with type=inline")?;
    start_inline_body_endpoints(
        inline_adapter,
        config.body.std_shell.enabled,
        config.body.std_shell.limits.clone(),
        config.body.std_web.enabled,
        config.body.std_web.limits.clone(),
    )?;

    let continuity = Arc::new(Mutex::new(
        ContinuityEngine::with_defaults_at(config.continuity.state_path.clone())
            .context("failed to initialize continuity engine")?,
    ));
    let afferent_rule_control: Arc<dyn AfferentRuleControlPort> =
        Arc::new(afferent_control.clone());
    let (efferent_producer, efferent_rx) =
        new_efferent_pathway(Some(config.cortex.outbox_capacity.max(1)));

    let cortex = Arc::new(Cortex::from_config(
        &config.cortex,
        config.r#loop.tick_interval_ms,
        Arc::clone(&chat),
        None,
        Some(continuity.clone()),
        Some(afferent_rule_control.clone()),
        Some(efferent_producer.clone()),
    ));

    let (tick_grant_tx, tick_grant_rx) = mpsc::channel(config.cortex.inbox_capacity.max(1));

    let app_context = AppContext {
        lifecycle,
        shutdown: CancellationToken::new(),
        afferent_control,
        continuity: continuity.clone(),
        spine: spine_runtime.clone(),
    };

    let physical_state_reader: Arc<dyn PhysicalStateReadPort> =
        Arc::new(MainPhysicalStateReader { stem_state });

    let stem_tick_runtime = StemTickRuntime::new(
        StemDeps {
            tick_interval_ms: config.r#loop.tick_interval_ms,
            tick_grant_tx,
        },
        app_context.shutdown.child_token(),
    );
    let stem_task = tokio::spawn(
        async move { stem_tick_runtime.run().await }
            .instrument(tracing::info_span!(target: "core", "stem_tick_task")),
    );

    let efferent_task = spawn_efferent_runtime(
        efferent_rx,
        continuity.clone(),
        spine_runtime,
        stem_control,
        app_context.shutdown.child_token(),
        Duration::from_millis(config.r#loop.efferent_shutdown_drain_timeout_ms.max(1)),
    );

    let cortex_runtime = CortexRuntime::new(
        CortexDeps {
            tick_grant_rx,
            afferent_consumer,
            physical_state_reader,
            cortex_core: cortex,
        },
        app_context.shutdown.child_token(),
    );
    let cortex_task = tokio::spawn(
        async move { cortex_runtime.run().await }
            .instrument(tracing::info_span!(target: "core", "cortex_runtime_task")),
    );

    app_context.lifecycle.set(AppState::Running).await;

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
        "received_signal_starting_shutdown"
    );
    app_context.lifecycle.set(AppState::Closing).await;
    app_context.afferent_control.close_gate().await;
    app_context.shutdown.cancel();

    stem_task.await.context("stem tick task join failed")?;
    cortex_task
        .await
        .context("cortex runtime task join failed")?;
    efferent_task
        .await
        .context("efferent runtime task join failed")?;

    app_context.continuity.lock().await.flush()?;
    shutdown_global_spine(app_context.spine.clone()).await?;

    app_context.lifecycle.set(AppState::Closed).await;

    tracing::info!(
        target: "core",
        signal_name = signal_name,
        "core_runtime_stopped"
    );
    if let Err(err) = observability_runtime.shutdown() {
        eprintln!("WARN observability: opentelemetry_otlp_shutdown_failed error={err}");
    }
    Ok(())
}
