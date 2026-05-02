use std::{
    env, fs,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
use beluna::{
    ai_gateway::{
        chat::Chat,
        credentials::EnvCredentialProvider,
        types::{
            AIGatewayConfig, BackendDialect, BackendProfile, ChatConfig, CredentialRef,
            ModelProfile, ResilienceConfig,
        },
    },
    body::{
        SHELL_ACT_EXEC_ID, SHELL_ENDPOINT_NAME,
        payloads::{ShellLimits, WebLimits},
        shell::handle_shell_invoke,
        start_inline_body_endpoints,
    },
    config::{
        Config, CortexRuntimeConfig, InlineAdapterConfig, SpineAdapterConfig, SpineRuntimeConfig,
    },
    continuity::ContinuityEngine,
    cortex::Cortex,
    spine::{EndpointBinding, EndpointExecutionOutcome, Spine},
    stem::{
        AfferentRuleControlPort, SenseAfferentPathway, StemControlPort, StemPhysicalStateStore,
        new_efferent_pathway, spawn_efferent_runtime,
    },
    types::{Act, NeuralSignalDescriptor, NeuralSignalType, Sense},
};
use serde_json::{Value, json};
use tokio::{sync::Mutex, time::timeout};
use tokio_util::sync::CancellationToken;

use super::{
    ai::{AimockBoundary, render_fixture_tree},
    case::{AgentTaskCase, DescriptorSpec, EndpointSpec},
    endpoints::{AckRecordingEndpoint, TickClock},
    evidence::EvidenceJournal,
    o11y::{ContractEventCapture, summarize_ai_gateway_events},
    oracle::{OracleEngine, RunResult, assert_no_oracle_internal_error},
    workspace::{CaseWorkspace, FileTreeDiff, FileTreeSnapshot},
};

pub struct AgentTaskRunner {
    artifact_root: PathBuf,
}

enum RunMode<'a> {
    Replay,
    Live { config: &'a Config },
}

struct AiRuntime {
    mode_label: &'static str,
    config: AIGatewayConfig,
    metadata: Value,
    _aimock: Option<AimockBoundary>,
}

struct WorkingDirectoryGuard {
    previous: PathBuf,
}

impl AgentTaskRunner {
    pub fn new(artifact_root: PathBuf) -> Self {
        Self { artifact_root }
    }

    pub async fn run(&self, case: &AgentTaskCase) -> Result<RunResult> {
        self.run_with_mode(case, RunMode::Replay).await
    }

    pub async fn run_live(&self, case: &AgentTaskCase, config: &Config) -> Result<RunResult> {
        self.run_with_mode(case, RunMode::Live { config }).await
    }

    async fn run_with_mode(&self, case: &AgentTaskCase, mode: RunMode<'_>) -> Result<RunResult> {
        validate_case(case, &mode)?;
        let started_at = Instant::now();
        let run_id = uuid::Uuid::now_v7().to_string();
        let artifact_dir = self
            .artifact_root
            .join(mode_artifact_segment(&mode))
            .join(&case.id)
            .join(&run_id);
        fs::create_dir_all(&artifact_dir)
            .with_context(|| format!("failed to create artifact dir {}", artifact_dir.display()))?;

        let journal = EvidenceJournal::default();
        let o11y_capture = start_o11y_capture(&mode, &journal);
        let workspace = CaseWorkspace::create(&artifact_dir)?;
        workspace.materialize(&case.world.files)?;
        let _cwd_guard = bind_case_working_directory(case, &workspace, &journal)?;

        let tick_clock = TickClock::new(1);
        let (ingress, mut consumer, afferent_control) =
            SenseAfferentPathway::new_handles(16, 16, 16);
        let proprioception = case.world.proprioception.entries.clone();
        let stem_state = StemPhysicalStateStore::new(proprioception);
        let stem_control: Arc<dyn StemControlPort> = Arc::new(stem_state.clone());
        let spine = Spine::new(
            &spine_runtime_config(case),
            ingress.clone(),
            Arc::clone(&stem_control),
        );

        start_case_endpoints(case, &spine, &workspace, &journal, tick_clock.clone()).await?;
        let before_snapshot = workspace.snapshot()?;

        let ai_runtime = prepare_ai_runtime(case, &mode, &artifact_dir, &workspace).await?;
        journal.record("ai.boundary.started", ai_runtime.metadata.clone());

        let continuity_path = artifact_dir.join("continuity/state.json");
        let continuity = Arc::new(Mutex::new(ContinuityEngine::with_defaults_at(
            continuity_path,
        )?));
        let (producer, efferent_rx) = new_efferent_pathway(Some(case.runtime.max_acts.max(1) * 2));
        let shutdown = CancellationToken::new();
        let efferent_task = spawn_efferent_runtime(
            efferent_rx,
            Arc::clone(&continuity),
            Arc::clone(&spine),
            shutdown.clone(),
            Duration::from_millis(100),
        );

        let chat = Arc::new(Chat::new(
            &ai_runtime.config,
            Arc::new(EnvCredentialProvider),
        )?);
        let afferent_rule_control: Arc<dyn AfferentRuleControlPort> = Arc::new(afferent_control);
        let cortex = Cortex::from_config(
            &cortex_config(case, &mode),
            1,
            chat,
            None,
            Some(Arc::clone(&continuity)),
            Some(afferent_rule_control),
            Some(producer),
        );

        let mut pending_senses =
            vec![inject_and_admit_sense(case, &ingress, &mut consumer, &journal).await?];
        let max_ticks = case.runtime.max_ticks.max(1);
        let mut ticks_run = 0_u64;
        for tick in 1..=max_ticks {
            ticks_run = tick;
            tick_clock.set(tick);
            journal.record(
                "tick.started",
                json!({
                    "tick": tick,
                    "sense_count": pending_senses.len(),
                }),
            );
            let physical_state = stem_state.snapshot_for_cycle(tick).await;
            let output = timeout(
                Duration::from_millis(case.runtime.timeout_ms),
                cortex.cortex(&pending_senses, &physical_state),
            )
            .await
            .with_context(|| format!("timed out waiting for Cortex tick {tick}"))??;
            journal.record(
                "tick.completed",
                json!({
                    "tick": tick,
                    "pending_primary_continuation": output.pending_primary_continuation,
                    "ignore_all_trigger_for_ticks": output.control.ignore_all_trigger_for_ticks,
                }),
            );

            wait_for_file_expectations(case, &workspace).await?;
            pending_senses = drain_emitted_senses(&mut consumer, &journal, tick).await;
            let tick_snapshot = workspace.snapshot()?;
            let tick_diff = FileTreeDiff::between(&before_snapshot, &tick_snapshot);
            record_world_diff(&journal, "world.diff.tick", tick, &tick_diff)?;
            let tick_failures = OracleEngine::evaluate(case, &journal, &workspace, &tick_snapshot)?;
            assert_no_oracle_internal_error(&tick_failures)?;
            if tick_failures.is_empty() {
                journal.record(
                    "oracle.passed",
                    json!({
                        "tick": tick,
                    }),
                );
                break;
            }
        }

        shutdown.cancel();
        let _ = timeout(Duration::from_millis(500), efferent_task).await;
        let final_tick = ticks_run.max(1);
        let _ = drain_emitted_senses(&mut consumer, &journal, final_tick).await;
        let after_snapshot = workspace.snapshot()?;
        let diff = FileTreeDiff::between(&before_snapshot, &after_snapshot);
        record_world_diff(&journal, "world.diff", final_tick, &diff)?;

        let failures = OracleEngine::evaluate(case, &journal, &workspace, &after_snapshot)?;
        assert_no_oracle_internal_error(&failures)?;
        let passed = failures.is_empty();
        let contract_events = o11y_capture
            .as_ref()
            .map(ContractEventCapture::events)
            .unwrap_or_default();
        write_artifacts(&RunArtifacts {
            artifact_dir: artifact_dir.clone(),
            case,
            journal: &journal,
            passed,
            failures: &failures,
            elapsed: started_at.elapsed(),
            ai_mode: ai_runtime.mode_label,
            tick_count: ticks_run,
            before_snapshot: &before_snapshot,
            after_snapshot: &after_snapshot,
            diff: &diff,
            contract_events: &contract_events,
            write_o11y_artifacts: o11y_capture.is_some(),
        })?;

        Ok(RunResult {
            passed,
            failures,
            artifact_dir,
        })
    }
}

impl Drop for WorkingDirectoryGuard {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.previous);
    }
}

fn start_o11y_capture(
    mode: &RunMode<'_>,
    journal: &EvidenceJournal,
) -> Option<ContractEventCapture> {
    match mode {
        RunMode::Replay => None,
        RunMode::Live { .. } => {
            let capture = ContractEventCapture::start();
            journal.record(
                "o11y.capture.started",
                json!({
                    "source": "tracing.observability.contract",
                    "installed": capture.installed(),
                }),
            );
            Some(capture)
        }
    }
}

fn bind_case_working_directory(
    case: &AgentTaskCase,
    workspace: &CaseWorkspace,
    journal: &EvidenceJournal,
) -> Result<Option<WorkingDirectoryGuard>> {
    if !requires_std_shell(case) {
        return Ok(None);
    }

    let previous = env::current_dir().context("failed to read current working directory")?;
    env::set_current_dir(workspace.root()).with_context(|| {
        format!(
            "failed to bind case working directory to {}",
            workspace.root().display()
        )
    })?;
    journal.record(
        "world.cwd.bound",
        json!({
            "cwd": workspace.root().display().to_string(),
            "previous_cwd": previous.display().to_string(),
        }),
    );
    Ok(Some(WorkingDirectoryGuard { previous }))
}

async fn prepare_ai_runtime(
    case: &AgentTaskCase,
    mode: &RunMode<'_>,
    artifact_dir: &std::path::Path,
    workspace: &CaseWorkspace,
) -> Result<AiRuntime> {
    match mode {
        RunMode::Replay => {
            let rendered_fixtures = render_fixture_tree(
                &case.fixture_root(),
                &artifact_dir.join("aimock-fixtures"),
                &[
                    ("$CASE_WORKSPACE", workspace.root().display().to_string()),
                    ("$CASE_TMP", artifact_dir.display().to_string()),
                ],
            )?;
            let aimock = AimockBoundary::start(&rendered_fixtures).await?;
            let metadata = json!({
                "provider": "aimock",
                "mode": "replay",
                "base_url": aimock.base_url(),
                "origin_url": aimock.origin_url(),
                "fixtures_path": aimock.fixtures_path(),
            });
            Ok(AiRuntime {
                mode_label: "replay",
                config: replay_ai_gateway_config(case, aimock.base_url()),
                metadata,
                _aimock: Some(aimock),
            })
        }
        RunMode::Live { config } => Ok(AiRuntime {
            mode_label: "live",
            config: config.ai_gateway.clone(),
            metadata: json!({
                "provider": "configured_gateway",
                "mode": "live",
                "backend_count": config.ai_gateway.backends.len(),
            }),
            _aimock: None,
        }),
    }
}

async fn inject_and_admit_sense(
    case: &AgentTaskCase,
    ingress: &SenseAfferentPathway,
    consumer: &mut beluna::stem::SenseConsumerHandle,
    journal: &EvidenceJournal,
) -> Result<Sense> {
    let sense = Sense {
        sense_instance_id: uuid::Uuid::now_v7().to_string(),
        endpoint_id: case.task.injected_sense.endpoint_id.clone(),
        neural_signal_descriptor_id: case.task.injected_sense.neural_signal_descriptor_id.clone(),
        payload: case.task.injected_sense.payload.clone(),
        weight: case.task.injected_sense.weight,
        act_instance_id: None,
    };
    journal.record(
        "sense.injected",
        json!({
            "tick": 1,
            "sense_instance_id": sense.sense_instance_id,
            "endpoint_id": sense.endpoint_id,
            "neural_signal_descriptor_id": sense.neural_signal_descriptor_id,
            "payload": sense.payload,
            "weight": sense.weight,
        }),
    );
    ingress.send(sense).await?;

    let admitted_sense = timeout(
        Duration::from_millis(case.runtime.timeout_ms),
        consumer.recv(),
    )
    .await
    .context("timed out waiting for admitted sense")?
    .context("afferent consumer closed before admitting sense")?;
    journal.record(
        "sense.admitted",
        json!({
            "tick": 1,
            "sense_instance_id": admitted_sense.sense_instance_id,
            "endpoint_id": admitted_sense.endpoint_id,
            "neural_signal_descriptor_id": admitted_sense.neural_signal_descriptor_id,
        }),
    );
    Ok(admitted_sense)
}

async fn start_case_endpoints(
    case: &AgentTaskCase,
    spine: &Arc<Spine>,
    workspace: &CaseWorkspace,
    journal: &EvidenceJournal,
    tick_clock: TickClock,
) -> Result<()> {
    if requires_std_shell(case) {
        let inline_adapter = spine
            .inline_adapter()
            .context("std_shell endpoint requires an inline spine adapter")?;
        start_inline_body_endpoints(
            inline_adapter,
            true,
            shell_limits(),
            false,
            WebLimits::default(),
        )?;
        run_shell_preflight(workspace).await?;
        let runtime_endpoint_id = spine
            .body_endpoint_ids_snapshot()
            .into_iter()
            .find(|id| id.starts_with("shell."))
            .unwrap_or_else(|| "shell".to_string());
        journal.record(
            "endpoint.attached",
            json!({
                "endpoint_id": "shell",
                "runtime_endpoint_id": runtime_endpoint_id,
                "kind": "std_shell",
            }),
        );
    }

    for endpoint in &case.world.endpoints {
        match endpoint.kind.as_str() {
            "ack_recording_endpoint" => {
                attach_ack_endpoint(endpoint, spine, journal, tick_clock.clone()).await?
            }
            "std_shell" => {}
            other => bail!("unsupported endpoint driver kind '{}'", other),
        }
    }
    Ok(())
}

async fn run_shell_preflight(workspace: &CaseWorkspace) -> Result<()> {
    let probe = ".beluna-shell-preflight";
    let act = Act {
        act_instance_id: uuid::Uuid::now_v7().to_string(),
        endpoint_id: SHELL_ENDPOINT_NAME.to_string(),
        neural_signal_descriptor_id: SHELL_ACT_EXEC_ID.to_string(),
        might_emit_sense_ids: Vec::new(),
        payload: json!({
            "argv": ["/bin/sh", "-lc", format!("printf probe > {probe}")],
            "cwd": workspace.root().display().to_string(),
            "timeout_ms": 1000,
            "stdout_max_bytes": 1024,
            "stderr_max_bytes": 1024,
        }),
    };
    let output = handle_shell_invoke("agent-task-shell-preflight", &act, &shell_limits()).await;
    match output.outcome {
        EndpointExecutionOutcome::Applied { .. } => {}
        other => bail!("invalid_environment: shell preflight failed: {:?}", other),
    }
    let probe_path = workspace.root().join(probe);
    if !probe_path.exists() {
        bail!("invalid_environment: shell preflight did not create probe file");
    }
    fs::remove_file(&probe_path).with_context(|| {
        format!(
            "invalid_environment: failed to remove shell preflight probe {}",
            probe_path.display()
        )
    })?;
    Ok(())
}

async fn attach_ack_endpoint(
    endpoint: &EndpointSpec,
    spine: &Arc<Spine>,
    journal: &EvidenceJournal,
    tick_clock: TickClock,
) -> Result<()> {
    let response = endpoint
        .response
        .clone()
        .context("ack_recording_endpoint requires response spec")?;
    let driver = Arc::new(AckRecordingEndpoint::new(
        endpoint.id.clone(),
        response,
        journal.clone(),
        tick_clock,
    ));
    let handle = spine.add_endpoint(&endpoint.id, EndpointBinding::Inline(driver))?;
    let descriptors = endpoint
        .descriptors
        .iter()
        .map(|descriptor| descriptor_from_spec(&endpoint.id, descriptor))
        .collect::<Result<Vec<_>>>()?;
    let accepted = spine
        .add_ns_descriptors(&handle.body_endpoint_id, descriptors)
        .await?;
    if accepted.len() != endpoint.descriptors.len() {
        bail!(
            "endpoint '{}' descriptor registration incomplete: expected {}, accepted {}",
            endpoint.id,
            endpoint.descriptors.len(),
            accepted.len()
        );
    }
    journal.record(
        "endpoint.attached",
        json!({
            "endpoint_id": endpoint.id,
            "runtime_endpoint_id": handle.body_endpoint_id,
            "descriptor_count": accepted.len(),
        }),
    );
    Ok(())
}

async fn wait_for_file_expectations(case: &AgentTaskCase, workspace: &CaseWorkspace) -> Result<()> {
    if case.oracle.pass.files.is_empty() {
        return Ok(());
    }
    let deadline = Duration::from_millis(case.runtime.timeout_ms.min(2_000).max(1));
    let started = Instant::now();
    loop {
        let snapshot = workspace.snapshot()?;
        if workspace
            .evaluate_expectations(&case.oracle.pass.files, &snapshot)
            .is_empty()
        {
            return Ok(());
        }
        if started.elapsed() >= deadline {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn drain_emitted_senses(
    consumer: &mut beluna::stem::SenseConsumerHandle,
    journal: &EvidenceJournal,
    observed_after_tick: u64,
) -> Vec<Sense> {
    let mut emitted = Vec::new();
    let deadline = Instant::now() + Duration::from_millis(250);
    loop {
        match timeout(Duration::from_millis(25), consumer.recv()).await {
            Ok(Some(sense)) => {
                journal.record(
                    "sense.emitted",
                    json!({
                        "observed_after_tick": observed_after_tick,
                        "endpoint_id": &sense.endpoint_id,
                        "neural_signal_descriptor_id": &sense.neural_signal_descriptor_id,
                        "sense_instance_id": &sense.sense_instance_id,
                        "payload": &sense.payload,
                        "weight": sense.weight,
                        "act_instance_id": &sense.act_instance_id,
                    }),
                );
                emitted.push(sense);
            }
            _ if Instant::now() >= deadline => break,
            _ => {}
        }
    }
    emitted
}

fn record_world_diff(
    journal: &EvidenceJournal,
    stream: &str,
    tick: u64,
    diff: &FileTreeDiff,
) -> Result<()> {
    let mut value = serde_json::to_value(diff)?;
    if let Value::Object(map) = &mut value {
        map.insert("tick".to_string(), json!(tick));
    }
    journal.record(stream, value);
    Ok(())
}

fn descriptor_from_spec(
    endpoint_id: &str,
    descriptor: &DescriptorSpec,
) -> Result<NeuralSignalDescriptor> {
    let signal_type = match descriptor.signal_type.as_str() {
        "sense" => NeuralSignalType::Sense,
        "act" => NeuralSignalType::Act,
        other => bail!("unsupported neural signal type '{}'", other),
    };
    Ok(NeuralSignalDescriptor {
        r#type: signal_type,
        endpoint_id: endpoint_id.to_string(),
        neural_signal_descriptor_id: descriptor.neural_signal_descriptor_id.clone(),
        payload_schema: descriptor.payload_schema.clone(),
    })
}

fn replay_ai_gateway_config(case: &AgentTaskCase, base_url: &str) -> AIGatewayConfig {
    AIGatewayConfig {
        backends: vec![BackendProfile {
            id: "agent-task-aimock".to_string(),
            dialect: BackendDialect::OpenAiCompatible,
            endpoint: Some(base_url.to_string()),
            credential: CredentialRef::None,
            models: vec![ModelProfile {
                id: case.ai.model.clone(),
                aliases: vec!["default".to_string()],
            }],
            capabilities: None,
            copilot: None,
        }],
        chat: ChatConfig::default(),
        resilience: ResilienceConfig::default(),
    }
}

fn cortex_config(case: &AgentTaskCase, mode: &RunMode<'_>) -> CortexRuntimeConfig {
    let mut config = match mode {
        RunMode::Replay => CortexRuntimeConfig::default(),
        RunMode::Live { config } => config.cortex.clone(),
    };
    config.default_limits.max_primary_turns_per_tick = case.runtime.max_primary_turns.max(1);
    config.default_limits.max_cycle_time_ms = case.runtime.timeout_ms.max(1);
    config
}

fn spine_runtime_config(case: &AgentTaskCase) -> SpineRuntimeConfig {
    if requires_std_shell(case) {
        return SpineRuntimeConfig {
            adapters: vec![SpineAdapterConfig::Inline {
                config: InlineAdapterConfig::default(),
            }],
        };
    }
    SpineRuntimeConfig {
        adapters: Vec::new(),
    }
}

fn requires_std_shell(case: &AgentTaskCase) -> bool {
    case.world
        .endpoints
        .iter()
        .any(|endpoint| endpoint.kind == "std_shell")
}

fn shell_limits() -> ShellLimits {
    ShellLimits {
        default_timeout_ms: 1_000,
        max_timeout_ms: 5_000,
        default_stdout_max_bytes: 4 * 1024,
        max_stdout_max_bytes: 16 * 1024,
        default_stderr_max_bytes: 4 * 1024,
        max_stderr_max_bytes: 16 * 1024,
    }
}

fn validate_case(case: &AgentTaskCase, mode: &RunMode<'_>) -> Result<()> {
    if case.schema_version != 0 {
        bail!(
            "unsupported agent task case schema_version={}",
            case.schema_version
        );
    }
    match mode {
        RunMode::Replay => {
            if case.ai.provider != "aimock" {
                bail!("unsupported replay AI provider '{}'", case.ai.provider);
            }
            if case.ai.mode != "replay" {
                bail!("unsupported replay AI mode '{}'", case.ai.mode);
            }
        }
        RunMode::Live { .. } => {}
    }
    if case.runtime.harness != "in_process" {
        bail!("unsupported harness '{}'", case.runtime.harness);
    }
    if case.runtime.tick_source != "manual" {
        bail!("unsupported tick_source '{}'", case.runtime.tick_source);
    }
    Ok(())
}

fn mode_artifact_segment(mode: &RunMode<'_>) -> &'static str {
    match mode {
        RunMode::Replay => "replay",
        RunMode::Live { .. } => "live",
    }
}

struct RunArtifacts<'a> {
    artifact_dir: PathBuf,
    case: &'a AgentTaskCase,
    journal: &'a EvidenceJournal,
    passed: bool,
    failures: &'a [String],
    elapsed: Duration,
    ai_mode: &'static str,
    tick_count: u64,
    before_snapshot: &'a FileTreeSnapshot,
    after_snapshot: &'a FileTreeSnapshot,
    diff: &'a FileTreeDiff,
    contract_events: &'a [Value],
    write_o11y_artifacts: bool,
}

fn write_artifacts(input: &RunArtifacts<'_>) -> Result<()> {
    let events = input.journal.events();
    let mut evidence_jsonl = String::new();
    for event in &events {
        evidence_jsonl.push_str(&serde_json::to_string(event)?);
        evidence_jsonl.push('\n');
    }
    fs::write(input.artifact_dir.join("evidence.jsonl"), evidence_jsonl)?;
    fs::write(
        input.artifact_dir.join("world-before.json"),
        serde_json::to_vec_pretty(input.before_snapshot)?,
    )?;
    fs::write(
        input.artifact_dir.join("world-after.json"),
        serde_json::to_vec_pretty(input.after_snapshot)?,
    )?;
    fs::write(
        input.artifact_dir.join("world-diff.json"),
        serde_json::to_vec_pretty(input.diff)?,
    )?;
    if input.write_o11y_artifacts {
        let mut contract_jsonl = String::new();
        for event in input.contract_events {
            contract_jsonl.push_str(&serde_json::to_string(event)?);
            contract_jsonl.push('\n');
        }
        fs::write(
            input.artifact_dir.join("o11y-contract-events.jsonl"),
            contract_jsonl,
        )?;
        fs::write(
            input.artifact_dir.join("ai-gateway-summary.json"),
            serde_json::to_vec_pretty(&summarize_ai_gateway_events(input.contract_events)?)?,
        )?;
    }
    fs::write(
        input.artifact_dir.join("result.json"),
        serde_json::to_vec_pretty(&json!({
            "case_id": input.case.id,
            "title": input.case.title,
            "success_claim": input.case.task.success_claim,
            "ai_mode": input.ai_mode,
            "passed": input.passed,
            "failures": input.failures,
            "wall_time_ms": input.elapsed.as_millis() as u64,
            "tick_count": input.tick_count,
            "event_count": events.len(),
            "o11y_contract_event_count": input.contract_events.len(),
        }))?,
    )?;
    Ok(())
}
