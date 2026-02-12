use std::{
    collections::{BTreeMap, VecDeque},
    fs,
    io::ErrorKind,
    os::unix::fs::FileTypeExt,
    path::Path,
    sync::Arc,
};

use anyhow::{Context, Result, bail};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::{UnixListener, UnixStream},
    signal::unix::{SignalKind, signal},
    sync::mpsc,
};

use crate::{
    admission::{AdmissionDisposition, AdmissionReport},
    ai_gateway::{credentials::EnvCredentialProvider, gateway::AIGateway, telemetry::NoopTelemetrySink},
    config::Config,
    continuity::ContinuityEngine,
    cortex::{
        AIGatewayAttemptExtractor, AIGatewayPayloadFiller, AIGatewayPrimaryReasoner,
        AdmissionOutcomeSignal, CapabilityCatalog, CortexPipeline, CortexReactor,
        DeterministicAttemptClamp, EndpointSnapshot, IntentContext, NoopTelemetryPort,
        ReactionInput, ReactionLimits, SenseDelta,
    },
    protocol::{ClientMessage, parse_client_message},
};

const MAX_FEEDBACK_WINDOW: usize = 64;

enum ExitReason {
    SocketMessage,
    Signal(&'static str),
}

#[derive(Default)]
struct CortexIngressAssembler {
    reaction_id: u64,
    capability_catalog: CapabilityCatalog,
    limits: ReactionLimits,
    context: IntentContext,
    env_snapshots: BTreeMap<String, EndpointSnapshot>,
    admission_feedback: VecDeque<AdmissionOutcomeSignal>,
}

impl CortexIngressAssembler {
    fn with_defaults(limits: ReactionLimits) -> Self {
        Self {
            limits,
            ..Self::default()
        }
    }

    fn on_message(&mut self, message: ClientMessage) -> Option<ReactionInput> {
        match message {
            ClientMessage::Exit => None,
            ClientMessage::Sense(sense) => Some(self.build_reaction_input(sense)),
            ClientMessage::EnvSnapshot(snapshot) => {
                self.env_snapshots
                    .insert(snapshot.endpoint_key.clone(), snapshot);
                None
            }
            ClientMessage::AdmissionFeedback(feedback) => {
                self.admission_feedback.push_back(feedback);
                while self.admission_feedback.len() > MAX_FEEDBACK_WINDOW {
                    self.admission_feedback.pop_front();
                }
                None
            }
            ClientMessage::CapabilityCatalogUpdate(catalog) => {
                self.capability_catalog = catalog;
                None
            }
            ClientMessage::CortexLimitsUpdate(limits) => {
                self.limits = limits;
                None
            }
            ClientMessage::IntentContextUpdate(context) => {
                self.context = context;
                None
            }
        }
    }

    fn build_reaction_input(&mut self, sense: SenseDelta) -> ReactionInput {
        self.reaction_id = self.reaction_id.saturating_add(1);
        ReactionInput {
            reaction_id: self.reaction_id,
            sense_window: vec![sense],
            env_snapshots: self.env_snapshots.values().cloned().collect(),
            admission_feedback: self.admission_feedback.iter().cloned().collect(),
            capability_catalog: self.capability_catalog.clone(),
            limits: self.limits.clone(),
            context: self.context.clone(),
        }
    }
}

pub async fn run(config: Config) -> Result<()> {
    prepare_socket_path(&config.socket_path)?;
    let listener = UnixListener::bind(&config.socket_path)
        .with_context(|| format!("unable to bind socket {}", config.socket_path.display()))?;

    let gateway = Arc::new(
        AIGateway::new(
            config.ai_gateway.clone(),
            Arc::new(EnvCredentialProvider),
            Arc::new(NoopTelemetrySink),
        )
        .context("failed to construct ai gateway for cortex reactor")?,
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

    let pipeline = CortexPipeline::new(primary, extractor, filler, clamp, telemetry);
    let reactor = CortexReactor::new(pipeline);

    let (reaction_tx, reaction_rx) = mpsc::channel::<ReactionInput>(config.cortex.inbox_capacity);
    let (result_tx, mut result_rx) =
        mpsc::channel::<crate::cortex::ReactionResult>(config.cortex.outbox_capacity);

    let reactor_task = tokio::spawn(async move {
        reactor.run(reaction_rx, result_tx).await;
    });

    let mut sigint =
        signal(SignalKind::interrupt()).context("unable to listen for SIGINT (Ctrl+C)")?;
    let mut sigterm = signal(SignalKind::terminate()).context("unable to listen for SIGTERM")?;
    let (message_tx, mut message_rx) = mpsc::unbounded_channel::<ClientMessage>();

    let mut assembler = CortexIngressAssembler::with_defaults(config.cortex.default_limits.clone());
    let mut continuity = ContinuityEngine::with_defaults(1_000_000);

    eprintln!(
        "Beluna listening on unix socket (NDJSON): {}",
        config.socket_path.display()
    );

    let exit_reason = loop {
        tokio::select! {
            _ = sigint.recv() => break ExitReason::Signal("SIGINT"),
            _ = sigterm.recv() => break ExitReason::Signal("SIGTERM"),
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((stream, _)) => {
                        let sender = message_tx.clone();
                        tokio::spawn(async move {
                            if let Err(err) = handle_client(stream, sender).await {
                                eprintln!("client handling failed: {err:#}");
                            }
                        });
                    }
                    Err(err) => eprintln!("accept failed: {err}"),
                }
            }
            Some(message) = message_rx.recv() => {
                if matches!(message, ClientMessage::Exit) {
                    break ExitReason::SocketMessage;
                }
                if let Some(input) = assembler.on_message(message) {
                    match reaction_tx.try_send(input) {
                        Ok(()) => {}
                        Err(mpsc::error::TrySendError::Full(_)) => {
                            eprintln!("cortex inbox is full; dropping reaction input due to backpressure");
                        }
                        Err(mpsc::error::TrySendError::Closed(_)) => {
                            eprintln!("cortex inbox is closed; stopping runtime");
                            break ExitReason::Signal("REACTOR_CLOSED");
                        }
                    }
                }
            }
            Some(result) = result_rx.recv() => {
                let reaction_id = result.reaction_id;
                match continuity.process_attempts(reaction_id, result.attempts) {
                    Ok(cycle_output) => {
                        let feedback = admission_report_to_feedback(&cycle_output.admission_report);
                        for signal in feedback {
                            let _ = assembler.on_message(ClientMessage::AdmissionFeedback(signal));
                        }

                        let mut denied = 0usize;
                        for outcome in &cycle_output.admission_report.outcomes {
                            if !matches!(outcome.disposition, AdmissionDisposition::Admitted { .. }) {
                                denied = denied.saturating_add(1);
                            }
                        }

                        eprintln!(
                            "reaction {} bridged: attempts={} admitted={} denied={} external_debits={} expired={}",
                            reaction_id,
                            cycle_output.admission_report.outcomes.len(),
                            cycle_output.admitted_action_count,
                            denied,
                            cycle_output.external_debit_applied_count,
                            cycle_output.expired_reservation_count
                        );
                    }
                    Err(err) => {
                        eprintln!(
                            "continuity bridge failed for reaction {}: {}",
                            reaction_id, err
                        );
                    }
                }
            }
        }
    };

    drop(reaction_tx);
    let _ = reactor_task.await;

    cleanup_socket_path(&config.socket_path)?;
    match exit_reason {
        ExitReason::SocketMessage => eprintln!("Beluna stopped: received exit message"),
        ExitReason::Signal(signal_name) => eprintln!("Beluna stopped: received {signal_name}"),
    }

    Ok(())
}

async fn handle_client(stream: UnixStream, message_tx: mpsc::UnboundedSender<ClientMessage>) -> Result<()> {
    let mut lines = BufReader::new(stream).lines();

    while let Some(line) = lines.next_line().await? {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match parse_client_message(line) {
            Ok(message) => {
                let _ = message_tx.send(message);
            }
            Err(err) => eprintln!("ignoring invalid protocol message: {err}"),
        }
    }

    Ok(())
}

fn prepare_socket_path(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("unable to create {}", parent.display()))?;
    }

    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_socket() || metadata.is_file() {
                fs::remove_file(path)
                    .with_context(|| format!("unable to remove stale socket {}", path.display()))?;
            } else {
                bail!(
                    "socket path exists but is not removable as file/socket: {}",
                    path.display()
                );
            }
        }
        Err(err) if err.kind() == ErrorKind::NotFound => {}
        Err(err) => {
            return Err(err).with_context(|| format!("unable to inspect {}", path.display()));
        }
    }

    Ok(())
}

fn cleanup_socket_path(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(_) => Ok(()),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err).with_context(|| format!("unable to remove {}", path.display())),
    }
}

fn admission_report_to_feedback(report: &AdmissionReport) -> Vec<AdmissionOutcomeSignal> {
    report
        .outcomes
        .iter()
        .map(|item| {
            let code = match &item.disposition {
                AdmissionDisposition::Admitted { degraded } => {
                    if *degraded {
                        "admitted_degraded".to_string()
                    } else {
                        "admitted".to_string()
                    }
                }
                AdmissionDisposition::DeniedHard { code } => code.clone(),
                AdmissionDisposition::DeniedEconomic { code } => code.clone(),
            };

            AdmissionOutcomeSignal {
                attempt_id: item.attempt_id.clone(),
                code,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::admission::{
        AdmissionDisposition, AdmissionReport, AdmissionReportItem, AdmissionWhy,
    };

    use super::admission_report_to_feedback;

    #[test]
    fn maps_admission_report_outcomes_to_feedback_signals() {
        let report = AdmissionReport {
            cycle_id: 1,
            outcomes: vec![
                AdmissionReportItem {
                    attempt_id: "att:1".to_string(),
                    disposition: AdmissionDisposition::Admitted { degraded: false },
                    why: None,
                    ledger_delta: None,
                    admitted_action_id: None,
                    degradation_profile_id: None,
                },
                AdmissionReportItem {
                    attempt_id: "att:2".to_string(),
                    disposition: AdmissionDisposition::Admitted { degraded: true },
                    why: None,
                    ledger_delta: None,
                    admitted_action_id: None,
                    degradation_profile_id: None,
                },
                AdmissionReportItem {
                    attempt_id: "att:3".to_string(),
                    disposition: AdmissionDisposition::DeniedHard {
                        code: "unknown_affordance".to_string(),
                    },
                    why: Some(AdmissionWhy::HardRule {
                        code: "unknown_affordance".to_string(),
                    }),
                    ledger_delta: None,
                    admitted_action_id: None,
                    degradation_profile_id: None,
                },
            ],
            total_reserved_survival_micro: 0,
        };

        let signals = admission_report_to_feedback(&report);
        assert_eq!(signals.len(), 3);
        assert_eq!(signals[0].attempt_id, "att:1");
        assert_eq!(signals[0].code, "admitted");
        assert_eq!(signals[1].code, "admitted_degraded");
        assert_eq!(signals[2].code, "unknown_affordance");
    }
}
