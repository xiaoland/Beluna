use std::{
    collections::{BTreeMap, VecDeque},
    sync::Arc,
    time::Duration,
};

use anyhow::{Context, Result};
use tokio::{
    signal::unix::{SignalKind, signal},
    sync::mpsc,
    time::MissedTickBehavior,
};
use tokio_util::sync::CancellationToken;

use crate::{
    admission::{
        AdmissionDisposition, AdmissionReport, AdmissionResolver, AdmissionResolverConfig,
        AffordanceProfile, AffordanceRegistry, CostAdmissionPolicy,
    },
    ai_gateway::{
        credentials::EnvCredentialProvider, gateway::AIGateway, telemetry::NoopTelemetrySink,
    },
    body::std::register_std_body_endpoints,
    config::Config,
    continuity::{
        ContinuityEngine, ContinuityState, NeuralSignalBatch, NoopDebitSource, SpinePortAdapter,
    },
    cortex::{
        AIGatewayAttemptExtractor, AIGatewayPayloadFiller, AIGatewayPrimaryReasoner,
        AdmissionOutcomeSignal, CapabilityCatalog, CortexPipeline, CortexReactor,
        DeterministicAttemptClamp, EndpointSnapshot, IntentContext, NoopTelemetryPort,
        ReactionInput, ReactionLimits, SenseDelta,
    },
    spine::{
        EndpointCapabilityDescriptor, EndpointPort, EndpointRegistration, EndpointRegistryPort,
        InMemoryEndpointRegistry, NativeFunctionEndpoint, RouteKey, RoutingSpineExecutor,
        SpineExecutionMode,
        adapters::{
            catalog_bridge::to_cortex_catalog,
            unix_socket::{BodyEndpointBroker, RemoteBodyEndpointPort, UnixSocketAdapter},
            wire::BodyIngressMessage,
        },
        types::CostVector,
    },
};

const MAX_FEEDBACK_WINDOW: usize = 64;
const REMOTE_ENDPOINT_TIMEOUT_MS: u64 = 30_000;

enum ExitReason {
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

    fn set_capability_catalog(&mut self, catalog: CapabilityCatalog) {
        self.capability_catalog = catalog;
    }

    fn on_message(&mut self, message: BodyIngressMessage) {
        match message {
            BodyIngressMessage::Sense(_) => {}
            BodyIngressMessage::EnvSnapshot(snapshot) => {
                self.env_snapshots
                    .insert(snapshot.endpoint_key.clone(), snapshot);
            }
            BodyIngressMessage::AdmissionFeedback(feedback) => {
                self.admission_feedback.push_back(feedback);
                while self.admission_feedback.len() > MAX_FEEDBACK_WINDOW {
                    self.admission_feedback.pop_front();
                }
            }
            BodyIngressMessage::CapabilityCatalogUpdate(_) => {
                // Spine owns capability catalog. Runtime catalog updates over wire are ignored.
            }
            BodyIngressMessage::CortexLimitsUpdate(limits) => {
                self.limits = limits;
            }
            BodyIngressMessage::IntentContextUpdate(context) => {
                self.context = context;
            }
            BodyIngressMessage::BodyEndpointRegister { .. }
            | BodyIngressMessage::BodyEndpointUnregister { .. }
            | BodyIngressMessage::BodyEndpointDisconnected { .. } => {}
        }
    }

    fn build_reaction_input(&mut self, sense_window: Vec<SenseDelta>) -> ReactionInput {
        self.reaction_id = self.reaction_id.saturating_add(1);
        ReactionInput {
            reaction_id: self.reaction_id,
            sense_window,
            env_snapshots: self.env_snapshots.values().cloned().collect(),
            admission_feedback: self.admission_feedback.iter().cloned().collect(),
            capability_catalog: self.capability_catalog.clone(),
            limits: self.limits.clone(),
            context: self.context.clone(),
        }
    }
}

pub async fn run(config: Config) -> Result<()> {
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

    let mut sigint =
        signal(SignalKind::interrupt()).context("unable to listen for SIGINT (Ctrl+C)")?;
    let mut sigterm = signal(SignalKind::terminate()).context("unable to listen for SIGTERM")?;
    let (message_tx, mut message_rx) = mpsc::unbounded_channel::<BodyIngressMessage>();
    let (builtin_sense_tx, mut builtin_sense_rx) = mpsc::unbounded_channel::<SenseDelta>();

    let registry: Arc<dyn EndpointRegistryPort> = Arc::new(InMemoryEndpointRegistry::new());
    register_default_native_endpoints(Arc::clone(&registry))?;
    register_std_body_endpoints(
        Arc::clone(&registry),
        builtin_sense_tx,
        config.body.std_shell.enabled,
        config.body.std_shell.limits.clone(),
        config.body.std_web.enabled,
        config.body.std_web.limits.clone(),
    )?;

    let spine_executor: Arc<dyn crate::spine::SpineExecutorPort> =
        Arc::new(RoutingSpineExecutor::new(
            SpineExecutionMode::SerializedDeterministic,
            Arc::clone(&registry),
        ));

    let admission = AdmissionResolver::new(
        default_affordance_registry(),
        CostAdmissionPolicy::default(),
        AdmissionResolverConfig::default(),
    );

    let mut continuity = ContinuityEngine::new(
        ContinuityState::new(1_000_000),
        admission,
        Arc::new(SpinePortAdapter::new(spine_executor)),
        Arc::new(NoopDebitSource),
    );

    let mut assembler = CortexIngressAssembler::with_defaults(config.cortex.default_limits.clone());
    assembler.set_capability_catalog(to_cortex_catalog(&continuity.capability_catalog_snapshot()));

    let body_endpoint_broker = Arc::new(BodyEndpointBroker::new(REMOTE_ENDPOINT_TIMEOUT_MS));

    let shutdown = CancellationToken::new();
    let unix_socket_adapter = UnixSocketAdapter::new(config.socket_path.clone());
    let adapter_shutdown = shutdown.clone();
    let adapter_broker = Arc::clone(&body_endpoint_broker);
    let adapter_task = tokio::spawn(async move {
        unix_socket_adapter
            .run(message_tx, adapter_broker, adapter_shutdown)
            .await
    });

    eprintln!(
        "Beluna listening on unix socket (NDJSON): {}",
        config.socket_path.display()
    );

    let mut ticker =
        tokio::time::interval(Duration::from_millis(config.r#loop.batch_window_ms.max(1)));
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let exit_reason = loop {
        tokio::select! {
            _ = sigint.recv() => break ExitReason::Signal("SIGINT"),
            _ = sigterm.recv() => break ExitReason::Signal("SIGTERM"),
            _ = ticker.tick() => {
                flush_cortex_batch(
                    &config,
                    &reactor,
                    &mut continuity,
                    &mut assembler,
                    true,
                ).await;
            }
            Some(sense) = builtin_sense_rx.recv() => {
                let dropped = continuity.enqueue_sense_delta(sense, config.r#loop.sense_queue_capacity);
                if dropped {
                    eprintln!("sense queue overflow; dropped oldest sense item");
                }
                flush_cortex_batch(
                    &config,
                    &reactor,
                    &mut continuity,
                    &mut assembler,
                    false,
                ).await;
            }
            Some(message) = message_rx.recv() => {
                match message {
                    BodyIngressMessage::BodyEndpointRegister { body_endpoint_id, endpoint_id, descriptor } => {
                        let route = descriptor.route.clone();
                        if let Err(err) = body_endpoint_broker.register_route(body_endpoint_id, &route) {
                            eprintln!("body endpoint register rejected: {err}");
                            continue;
                        }

                        let _ = registry.unregister(&route);

                        let endpoint: Arc<dyn EndpointPort> = Arc::new(RemoteBodyEndpointPort::new(
                            route.clone(),
                            Arc::clone(&body_endpoint_broker),
                        ));

                        if let Err(err) = registry.register(
                            EndpointRegistration {
                                endpoint_id,
                                descriptor,
                            },
                            endpoint,
                        ) {
                            body_endpoint_broker.unregister_route(body_endpoint_id, &route);
                            eprintln!("body endpoint route registration failed: {err}");
                            continue;
                        }

                        assembler.set_capability_catalog(
                            to_cortex_catalog(&continuity.capability_catalog_snapshot()),
                        );
                    }
                    BodyIngressMessage::BodyEndpointUnregister { body_endpoint_id, route } => {
                        body_endpoint_broker.unregister_route(body_endpoint_id, &route);
                        let _ = registry.unregister(&route);
                        assembler.set_capability_catalog(
                            to_cortex_catalog(&continuity.capability_catalog_snapshot()),
                        );
                    }
                    BodyIngressMessage::BodyEndpointDisconnected { body_endpoint_id, routes } => {
                        for route in routes {
                            body_endpoint_broker.unregister_route(body_endpoint_id, &route);
                            let _ = registry.unregister(&route);
                        }
                        assembler.set_capability_catalog(
                            to_cortex_catalog(&continuity.capability_catalog_snapshot()),
                        );
                    }
                    BodyIngressMessage::Sense(sense) => {
                        let dropped = continuity.enqueue_sense_delta(sense, config.r#loop.sense_queue_capacity);
                        if dropped {
                            eprintln!("sense queue overflow; dropped oldest sense item");
                        }

                        flush_cortex_batch(
                            &config,
                            &reactor,
                            &mut continuity,
                            &mut assembler,
                            false,
                        ).await;
                    }
                    other => {
                        assembler.on_message(other);
                    }
                }
            }
        }
    };

    shutdown.cancel();

    match adapter_task.await {
        Ok(Ok(())) => {}
        Ok(Err(err)) => eprintln!("unix socket adapter exited with error: {err:#}"),
        Err(err) => eprintln!("unix socket adapter task join failed: {err}"),
    }

    match exit_reason {
        ExitReason::Signal(signal_name) => eprintln!("Beluna stopped: received {signal_name}"),
    }

    Ok(())
}

async fn flush_cortex_batch(
    config: &Config,
    reactor: &CortexReactor,
    continuity: &mut ContinuityEngine,
    assembler: &mut CortexIngressAssembler,
    forced: bool,
) {
    let queued = continuity.sense_queue_len();
    if queued == 0 {
        return;
    }

    if !forced && queued < config.r#loop.batch_flush_sense_count.max(1) {
        return;
    }

    let sense_batch = continuity.dequeue_sense_batch(config.r#loop.batch_max_sense_count.max(1));
    if sense_batch.is_empty() {
        return;
    }

    assembler.set_capability_catalog(to_cortex_catalog(&continuity.capability_catalog_snapshot()));

    let reaction_input = assembler.build_reaction_input(sense_batch);
    let reaction_result = reactor.react_once(reaction_input).await;

    let dropped = continuity.enqueue_neural_signal_batch(
        NeuralSignalBatch {
            reaction_id: reaction_result.reaction_id,
            attempts: reaction_result.attempts,
        },
        config.r#loop.neural_signal_queue_capacity,
    );
    if dropped {
        eprintln!("neural signal queue overflow; dropped oldest signal batch");
    }

    while let Some(signal_batch) = continuity.pop_neural_signal_batch() {
        match continuity
            .process_attempts(signal_batch.reaction_id, signal_batch.attempts)
            .await
        {
            Ok(cycle_output) => {
                let feedback = admission_report_to_feedback(&cycle_output.admission_report);
                for signal in feedback {
                    assembler.on_message(BodyIngressMessage::AdmissionFeedback(signal));
                }

                let mut denied = 0usize;
                for outcome in &cycle_output.admission_report.outcomes {
                    if !matches!(outcome.disposition, AdmissionDisposition::Admitted { .. }) {
                        denied = denied.saturating_add(1);
                    }
                }

                eprintln!(
                    "reaction {} bridged: attempts={} admitted={} denied={} external_debits={} expired={}",
                    cycle_output.cycle_id,
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
                    signal_batch.reaction_id, err
                );
            }
        }
    }
}

fn register_default_native_endpoints(registry: Arc<dyn EndpointRegistryPort>) -> Result<()> {
    let native_endpoint = Arc::new(NativeFunctionEndpoint::new(Arc::new(|invocation| {
        let action = invocation.action;
        Ok(crate::spine::types::EndpointExecutionOutcome::Applied {
            actual_cost_micro: action.reserved_cost.survival_micro,
            reference_id: format!("native:settle:{}", action.action_id),
        })
    })));

    let register =
        |affordance_key: &str, capability_handle: &str, default_cost: CostVector| -> Result<()> {
            registry
                .register(
                    EndpointRegistration {
                        endpoint_id: format!("ep:native:{}:{}", affordance_key, capability_handle),
                        descriptor: EndpointCapabilityDescriptor {
                            route: RouteKey {
                                affordance_key: affordance_key.to_string(),
                                capability_handle: capability_handle.to_string(),
                            },
                            payload_schema: serde_json::json!({"type":"object"}),
                            max_payload_bytes: 16_384,
                            default_cost,
                            metadata: Default::default(),
                        },
                    },
                    native_endpoint.clone(),
                )
                .map_err(|err| anyhow::anyhow!(err.to_string()))?;
            Ok(())
        };

    register(
        "deliberate.plan",
        "cap.core",
        CostVector {
            survival_micro: 250,
            time_ms: 120,
            io_units: 1,
            token_units: 128,
        },
    )?;
    register(
        "deliberate.plan",
        "cap.core.lite",
        CostVector {
            survival_micro: 250,
            time_ms: 120,
            io_units: 1,
            token_units: 128,
        },
    )?;
    register(
        "deliberate.plan",
        "cap.core.minimal",
        CostVector {
            survival_micro: 250,
            time_ms: 120,
            io_units: 1,
            token_units: 128,
        },
    )?;
    register(
        "execute.tool",
        "cap.core",
        CostVector {
            survival_micro: 400,
            time_ms: 200,
            io_units: 2,
            token_units: 256,
        },
    )?;

    Ok(())
}

fn default_affordance_registry() -> AffordanceRegistry {
    AffordanceRegistry::new(vec![
        AffordanceProfile::default(),
        AffordanceProfile {
            profile_id: "default-execution".to_string(),
            affordance_key: "execute.tool".to_string(),
            capability_handle: "cap.core".to_string(),
            max_payload_bytes: 16_384,
            base_cost: CostVector {
                survival_micro: 400,
                time_ms: 200,
                io_units: 2,
                token_units: 256,
            },
            degradation_profiles: vec![],
        },
        AffordanceProfile {
            profile_id: "std-shell".to_string(),
            affordance_key: "tool.shell.exec".to_string(),
            capability_handle: "cap.std.shell".to_string(),
            max_payload_bytes: 65_536,
            base_cost: CostVector {
                survival_micro: 500,
                time_ms: 2_000,
                io_units: 4,
                token_units: 0,
            },
            degradation_profiles: vec![],
        },
        AffordanceProfile {
            profile_id: "std-web-fetch".to_string(),
            affordance_key: "tool.web.fetch".to_string(),
            capability_handle: "cap.std.web.fetch".to_string(),
            max_payload_bytes: 65_536,
            base_cost: CostVector {
                survival_micro: 450,
                time_ms: 2_500,
                io_units: 3,
                token_units: 0,
            },
            degradation_profiles: vec![],
        },
        AffordanceProfile {
            profile_id: "apple-chat-reply".to_string(),
            affordance_key: "chat.reply.emit".to_string(),
            capability_handle: "cap.apple.universal.chat".to_string(),
            max_payload_bytes: 32_768,
            base_cost: CostVector {
                survival_micro: 120,
                time_ms: 100,
                io_units: 1,
                token_units: 64,
            },
            degradation_profiles: vec![],
        },
    ])
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
