use std::{
    fs,
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
    config::{CortexRuntimeConfig, SpineRuntimeConfig},
    continuity::ContinuityEngine,
    cortex::Cortex,
    spine::{EndpointBinding, Spine},
    stem::{
        AfferentRuleControlPort, SenseAfferentPathway, StemControlPort, StemPhysicalStateStore,
        new_efferent_pathway, spawn_efferent_runtime,
    },
    types::{NeuralSignalDescriptor, NeuralSignalType, Sense},
};
use serde_json::json;
use tokio::{sync::Mutex, time::timeout};
use tokio_util::sync::CancellationToken;

use super::{
    ai::AimockBoundary,
    case::{AgentTaskCase, DescriptorSpec, EndpointSpec},
    endpoints::AckRecordingEndpoint,
    evidence::EvidenceJournal,
    oracle::{OracleEngine, RunResult, assert_no_oracle_internal_error},
};

pub struct AgentTaskRunner {
    artifact_root: PathBuf,
}

impl AgentTaskRunner {
    pub fn new(artifact_root: PathBuf) -> Self {
        Self { artifact_root }
    }

    pub async fn run(&self, case: &AgentTaskCase) -> Result<RunResult> {
        validate_case(case)?;
        let started_at = Instant::now();
        let run_id = uuid::Uuid::now_v7().to_string();
        let artifact_dir = self.artifact_root.join(&case.id).join(&run_id);
        fs::create_dir_all(&artifact_dir)
            .with_context(|| format!("failed to create artifact dir {}", artifact_dir.display()))?;

        let journal = EvidenceJournal::default();
        let aimock = AimockBoundary::start(&case.fixture_root()).await?;
        journal.record(
            "ai.boundary.started",
            json!({
                "provider": "aimock",
                "base_url": aimock.base_url(),
                "origin_url": aimock.origin_url(),
                "fixtures_path": aimock.fixtures_path(),
            }),
        );

        let (ingress, mut consumer, afferent_control) =
            SenseAfferentPathway::new_handles(16, 16, 16);
        let proprioception = case.world.proprioception.entries.clone();
        let stem_state = StemPhysicalStateStore::new(proprioception);
        let stem_control: Arc<dyn StemControlPort> = Arc::new(stem_state.clone());
        let spine = Spine::new(
            &SpineRuntimeConfig {
                adapters: Vec::new(),
            },
            ingress.clone(),
            Arc::clone(&stem_control),
        );

        attach_endpoints(case, &spine, &journal).await?;

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
            &ai_gateway_config(case, aimock.base_url()),
            Arc::new(EnvCredentialProvider),
        )?);
        let afferent_rule_control: Arc<dyn AfferentRuleControlPort> = Arc::new(afferent_control);
        let cortex = Cortex::from_config(
            &cortex_config(case),
            1,
            chat,
            None,
            Some(Arc::clone(&continuity)),
            Some(afferent_rule_control),
            Some(producer),
        );

        let sense = Sense {
            sense_instance_id: uuid::Uuid::now_v7().to_string(),
            endpoint_id: case.task.injected_sense.endpoint_id.clone(),
            neural_signal_descriptor_id: case
                .task
                .injected_sense
                .neural_signal_descriptor_id
                .clone(),
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

        let physical_state = stem_state.snapshot_for_cycle(1).await;
        timeout(
            Duration::from_millis(case.runtime.timeout_ms),
            cortex.cortex(&[admitted_sense], &physical_state),
        )
        .await
        .context("timed out waiting for Cortex cycle")??;

        shutdown.cancel();
        let _ = timeout(Duration::from_millis(500), efferent_task).await;

        let failures = OracleEngine::evaluate(case, &journal)?;
        assert_no_oracle_internal_error(&failures)?;
        let passed = failures.is_empty();
        write_artifacts(
            &artifact_dir,
            case,
            &journal,
            passed,
            &failures,
            started_at.elapsed(),
        )?;

        Ok(RunResult {
            passed,
            failures,
            artifact_dir,
        })
    }
}

async fn attach_endpoints(
    case: &AgentTaskCase,
    spine: &Arc<Spine>,
    journal: &EvidenceJournal,
) -> Result<()> {
    for endpoint in &case.world.endpoints {
        match endpoint.kind.as_str() {
            "ack_recording_endpoint" => attach_ack_endpoint(endpoint, spine, journal).await?,
            other => bail!("unsupported endpoint driver kind '{}'", other),
        }
    }
    Ok(())
}

async fn attach_ack_endpoint(
    endpoint: &EndpointSpec,
    spine: &Arc<Spine>,
    journal: &EvidenceJournal,
) -> Result<()> {
    let driver = Arc::new(AckRecordingEndpoint::new(
        endpoint.id.clone(),
        endpoint.response.clone(),
        journal.clone(),
        1,
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

fn cortex_config(case: &AgentTaskCase) -> CortexRuntimeConfig {
    let mut config = CortexRuntimeConfig::default();
    config.default_limits.max_primary_turns_per_tick = case.runtime.max_primary_turns.max(1);
    config.default_limits.max_cycle_time_ms = case.runtime.timeout_ms.max(1);
    config
}

fn ai_gateway_config(case: &AgentTaskCase, base_url: &str) -> AIGatewayConfig {
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

fn validate_case(case: &AgentTaskCase) -> Result<()> {
    if case.schema_version != 0 {
        bail!(
            "unsupported agent task case schema_version={}",
            case.schema_version
        );
    }
    if case.ai.provider != "aimock" {
        bail!("unsupported AI provider '{}'", case.ai.provider);
    }
    if case.ai.mode != "replay" {
        bail!("unsupported AI mode '{}'", case.ai.mode);
    }
    if case.runtime.harness != "in_process" {
        bail!("unsupported harness '{}'", case.runtime.harness);
    }
    if case.runtime.tick_source != "manual" {
        bail!("unsupported tick_source '{}'", case.runtime.tick_source);
    }
    Ok(())
}

fn write_artifacts(
    artifact_dir: &PathBuf,
    case: &AgentTaskCase,
    journal: &EvidenceJournal,
    passed: bool,
    failures: &[String],
    elapsed: Duration,
) -> Result<()> {
    let events = journal.events();
    let mut evidence_jsonl = String::new();
    for event in &events {
        evidence_jsonl.push_str(&serde_json::to_string(event)?);
        evidence_jsonl.push('\n');
    }
    fs::write(artifact_dir.join("evidence.jsonl"), evidence_jsonl)?;
    fs::write(
        artifact_dir.join("result.json"),
        serde_json::to_vec_pretty(&json!({
            "case_id": case.id,
            "title": case.title,
            "success_claim": case.task.success_claim,
            "passed": passed,
            "failures": failures,
            "wall_time_ms": elapsed.as_millis() as u64,
            "event_count": events.len(),
        }))?,
    )?;
    Ok(())
}
