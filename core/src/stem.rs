use std::{
    collections::{BTreeMap, VecDeque},
    sync::Arc,
    time::Duration,
};

use anyhow::Result;
use tokio::{
    sync::{Mutex, mpsc},
    task::JoinHandle,
    time::{Instant, MissedTickBehavior},
};

use crate::{
    afferent_pathway::SenseAfferentPathway,
    config::TickMissedBehavior,
    continuity::{ContinuityEngine, DispatchContext as ContinuityDispatchContext},
    cortex::Cortex,
    spine::{ActDispatchResult, Spine},
    types::{
        Act, DispatchDecision, NeuralSignalDescriptor, NeuralSignalDescriptorCatalog,
        NeuralSignalDescriptorRouteKey, NeuralSignalType, PhysicalLedgerSnapshot, PhysicalState,
        ProprioceptionDropPatch, ProprioceptionPatch, Sense,
    },
};

enum StemMode {
    Active,
    SleepingUntil(Instant),
}

struct CycleOutcome {
    sleep_deadline: Option<Instant>,
    wait_for_sense: bool,
}

struct DispatchTask {
    act: Act,
    cycle_id: u64,
    seq_no: u64,
}

const DISPATCH_QUEUE_CAPACITY: usize = 128;
const DISPATCH_TERMINAL_RETENTION_LIMIT: usize = 128;

pub struct Stem {
    cycle_id: u64,
    cortex: Arc<Cortex>,
    continuity: Arc<Mutex<ContinuityEngine>>,
    spine: Arc<Spine>,
    afferent_pathway: SenseAfferentPathway,
    sense_rx: mpsc::Receiver<Sense>,
    tick_interval_ms: u64,
    tick_missed_behavior: TickMissedBehavior,
    main_startup_proprioception: BTreeMap<String, String>,
    dynamic_proprioception: BTreeMap<String, String>,
    dispatch_tx: Option<mpsc::Sender<DispatchTask>>,
    dispatch_task: Option<JoinHandle<()>>,
}

impl Stem {
    pub fn new(
        cortex: Arc<Cortex>,
        continuity: Arc<Mutex<ContinuityEngine>>,
        spine: Arc<Spine>,
        afferent_pathway: SenseAfferentPathway,
        sense_rx: mpsc::Receiver<Sense>,
        tick_interval_ms: u64,
        tick_missed_behavior: TickMissedBehavior,
        main_startup_proprioception: BTreeMap<String, String>,
    ) -> Self {
        Self {
            cycle_id: 0,
            cortex,
            continuity,
            spine,
            afferent_pathway,
            sense_rx,
            tick_interval_ms: tick_interval_ms.max(1),
            tick_missed_behavior,
            main_startup_proprioception,
            dynamic_proprioception: BTreeMap::new(),
            dispatch_tx: None,
            dispatch_task: None,
        }
    }

    #[tracing::instrument(name = "stem_run", target = "stem", skip(self))]
    pub async fn run(mut self) -> Result<()> {
        self.start_dispatch_worker();
        let mut mode = StemMode::Active;
        let mut wait_for_sense = false;
        let mut tick = tokio::time::interval(Duration::from_millis(self.tick_interval_ms));
        tick.set_missed_tick_behavior(match self.tick_missed_behavior {
            TickMissedBehavior::Skip => MissedTickBehavior::Skip,
        });

        loop {
            match mode {
                StemMode::Active => {
                    let cycle_outcome = if wait_for_sense {
                        let Some(first_sense) = self.sense_rx.recv().await else {
                            break;
                        };
                        if matches!(first_sense, Sense::Hibernate) {
                            break;
                        }

                        let mut senses = vec![first_sense];
                        senses.extend(self.drain_senses_nonblocking());
                        if senses.iter().any(|sense| matches!(sense, Sense::Hibernate)) {
                            break;
                        }

                        let cycle_outcome = self.execute_cycle(senses).await?;
                        tick.reset();
                        cycle_outcome
                    } else {
                        tick.tick().await;

                        let senses = self.drain_senses_nonblocking();
                        if senses.iter().any(|sense| matches!(sense, Sense::Hibernate)) {
                            break;
                        }

                        self.execute_cycle(senses).await?
                    };

                    wait_for_sense = cycle_outcome.wait_for_sense;
                    if let Some(deadline) = cycle_outcome.sleep_deadline {
                        mode = StemMode::SleepingUntil(deadline);
                    }
                }
                StemMode::SleepingUntil(deadline) => {
                    let now = Instant::now();
                    if now >= deadline {
                        let cycle_outcome = self.execute_cycle(Vec::new()).await?;
                        tick.reset();
                        wait_for_sense = cycle_outcome.wait_for_sense;
                        if let Some(next_deadline) = cycle_outcome.sleep_deadline {
                            mode = StemMode::SleepingUntil(next_deadline);
                        } else {
                            mode = StemMode::Active;
                        }
                        continue;
                    }

                    let wait = deadline.saturating_duration_since(now);
                    tokio::select! {
                        _ = tokio::time::sleep(wait) => {
                            let cycle_outcome = self.execute_cycle(Vec::new()).await?;
                            tick.reset();
                            wait_for_sense = cycle_outcome.wait_for_sense;
                            if let Some(next_deadline) = cycle_outcome.sleep_deadline {
                                mode = StemMode::SleepingUntil(next_deadline);
                            } else {
                                mode = StemMode::Active;
                            }
                        }
                        maybe_sense = self.sense_rx.recv() => {
                            let Some(first_sense) = maybe_sense else {
                                break;
                            };
                            if matches!(first_sense, Sense::Hibernate) {
                                break;
                            }

                            let mut senses = vec![first_sense];
                            senses.extend(self.drain_senses_nonblocking());
                            if senses.iter().any(|sense| matches!(sense, Sense::Hibernate)) {
                                break;
                            }

                            let cycle_outcome = self.execute_cycle(senses).await?;
                            tick.reset();
                            wait_for_sense = cycle_outcome.wait_for_sense;
                            if let Some(next_deadline) = cycle_outcome.sleep_deadline {
                                mode = StemMode::SleepingUntil(next_deadline);
                            } else {
                                mode = StemMode::Active;
                            }
                        }
                    }
                }
            }
        }

        self.shutdown_dispatch_worker().await;
        Ok(())
    }

    async fn execute_cycle(&mut self, sense_batch: Vec<Sense>) -> Result<CycleOutcome> {
        for sense in &sense_batch {
            match sense {
                Sense::NewNeuralSignalDescriptors(patch) => {
                    self.continuity
                        .lock()
                        .await
                        .apply_neural_signal_descriptor_patch(patch);
                }
                Sense::DropNeuralSignalDescriptors(drop_patch) => {
                    self.continuity
                        .lock()
                        .await
                        .apply_neural_signal_descriptor_drop(drop_patch);
                }
                Sense::NewProprioceptions(patch) => {
                    self.apply_proprioception_patch(patch);
                }
                Sense::DropProprioceptions(drop_patch) => {
                    self.apply_proprioception_drop(drop_patch);
                }
                Sense::Domain(_) | Sense::Hibernate => {}
            }
        }

        self.cycle_id = self.cycle_id.saturating_add(1);

        let domain_senses: Vec<_> = sense_batch
            .iter()
            .filter_map(|sense| match sense {
                Sense::Domain(_) => Some(sense.clone()),
                _ => None,
            })
            .collect();

        let cognition_state = self.continuity.lock().await.cognition_state_snapshot();
        let physical_state = self.compose_physical_state(self.cycle_id).await?;

        let output = match self
            .cortex
            .cortex(&domain_senses, &physical_state, &cognition_state)
            .await
        {
            Ok(output) => output,
            Err(err) => {
                tracing::warn!(
                    target: "stem",
                    cycle_id = self.cycle_id,
                    error = %err,
                    "cortex_failed_for_cycle"
                );
                return Ok(CycleOutcome {
                    sleep_deadline: None,
                    wait_for_sense: false,
                });
            }
        };
        let wait_for_sense = output.wait_for_sense;

        self.continuity
            .lock()
            .await
            .persist_cognition_state(output.new_cognition_state)?;

        tracing::debug!(
            target: "stem",
            cycle_id = self.cycle_id,
            generated_acts = output.acts.len(),
            "cycle_generated_acts"
        );

        let mut sleep_deadline = None;
        for (index, act) in output.acts.into_iter().enumerate() {
            let seq_no = (index as u64) + 1;
            if let Some(deadline) = self.try_handle_sleep_act(&act) {
                sleep_deadline = Some(deadline);
                tracing::info!(
                    target: "stem",
                    cycle_id = self.cycle_id,
                    seq_no = seq_no,
                    act_instance_id = %act.act_instance_id,
                    "sleep_act_intercepted"
                );
                break;
            }

            if let Some(dispatch_tx) = &self.dispatch_tx {
                if dispatch_tx
                    .send(DispatchTask {
                        act: act.clone(),
                        cycle_id: self.cycle_id,
                        seq_no,
                    })
                    .await
                    .is_err()
                {
                    tracing::warn!(
                        target: "stem.dispatch",
                        cycle_id = self.cycle_id,
                        seq_no = seq_no,
                        act_instance_id = %act.act_instance_id,
                        "dispatch_worker_queue_closed"
                    );
                }
            } else {
                tracing::warn!(
                    target: "stem.dispatch",
                    cycle_id = self.cycle_id,
                    seq_no = seq_no,
                    act_instance_id = %act.act_instance_id,
                    "dispatch_worker_not_initialized"
                );
            }
        }

        Ok(CycleOutcome {
            sleep_deadline,
            wait_for_sense,
        })
    }

    fn drain_senses_nonblocking(&mut self) -> Vec<Sense> {
        let mut drained = Vec::new();
        while let Ok(next) = self.sense_rx.try_recv() {
            drained.push(next);
        }
        drained
    }

    fn try_handle_sleep_act(&self, act: &Act) -> Option<Instant> {
        if act.endpoint_id != "core.control" || act.neural_signal_descriptor_id != "sleep" {
            return None;
        }

        let seconds = act.payload.get("seconds")?.as_u64()?.max(1);
        Some(Instant::now() + Duration::from_secs(seconds))
    }

    fn apply_proprioception_patch(&mut self, patch: &ProprioceptionPatch) {
        for (key, value) in &patch.entries {
            self.dynamic_proprioception
                .insert(key.clone(), value.clone());
        }
    }

    fn apply_proprioception_drop(&mut self, drop_patch: &ProprioceptionDropPatch) {
        for key in &drop_patch.keys {
            self.dynamic_proprioception.remove(key);
        }
    }

    #[tracing::instrument(
        name = "stem_compose_physical_state",
        target = "stem",
        skip(self),
        fields(cycle_id = cycle_id)
    )]
    async fn compose_physical_state(&self, cycle_id: u64) -> Result<PhysicalState> {
        let spine_catalog = self.spine.neural_signal_descriptor_catalog_snapshot();
        let continuity_catalog = self
            .continuity
            .lock()
            .await
            .neural_signal_descriptor_snapshot();
        let stem_catalog = stem_control_descriptor_catalog();
        let merged = merge_neural_signal_descriptor_catalogs(
            cycle_id,
            spine_catalog,
            continuity_catalog,
            stem_catalog,
        );

        Ok(PhysicalState {
            cycle_id,
            ledger: PhysicalLedgerSnapshot::default(),
            capabilities: merged,
            proprioception: self.compose_proprioception(),
        })
    }

    fn compose_proprioception(&self) -> BTreeMap<String, String> {
        let mut merged = self.main_startup_proprioception.clone();
        for (key, value) in &self.dynamic_proprioception {
            merged.insert(key.clone(), value.clone());
        }
        merged
    }

    fn start_dispatch_worker(&mut self) {
        if self.dispatch_tx.is_some() {
            return;
        }

        let (dispatch_tx, mut dispatch_rx) = mpsc::channel::<DispatchTask>(DISPATCH_QUEUE_CAPACITY);
        let spine = Arc::clone(&self.spine);
        let continuity = Arc::clone(&self.continuity);
        let afferent_pathway = self.afferent_pathway.clone();
        let dispatch_task = tokio::spawn(async move {
            let mut terminal_status_keys = VecDeque::new();
            while let Some(task) = dispatch_rx.recv().await {
                let status_key = dispatch_status_key(&task.act.act_instance_id);
                emit_status_patch(&afferent_pathway, &status_key, "DISPATCHING").await;

                let continuity_status = match continuity
                    .lock()
                    .await
                    .on_act(
                        &task.act,
                        &ContinuityDispatchContext {
                            cycle_id: task.cycle_id,
                            act_seq_no: task.seq_no,
                        },
                    ) {
                    Ok(DispatchDecision::Continue) => None,
                    Ok(DispatchDecision::Break) => Some("REJECTED"),
                    Err(err) => {
                        tracing::warn!(
                            target: "stem.dispatch",
                            cycle_id = task.cycle_id,
                            seq_no = task.seq_no,
                            act_instance_id = %task.act.act_instance_id,
                            error = %err,
                            "continuity_dispatch_failed_mark_lost"
                        );
                        Some("LOST")
                    }
                };

                let terminal_status = if let Some(status) = continuity_status {
                    status
                } else {
                    match spine.on_act_final(task.act.clone()).await {
                        Ok(ActDispatchResult::Acknowledged { .. }) => "ACK",
                        Ok(ActDispatchResult::Rejected { .. }) => "REJECTED",
                        Ok(ActDispatchResult::Lost { .. }) => "LOST",
                        Err(err) => {
                            tracing::warn!(
                                target: "stem.dispatch",
                                cycle_id = task.cycle_id,
                                seq_no = task.seq_no,
                                act_instance_id = %task.act.act_instance_id,
                                error = %err,
                                "spine_dispatch_failed_mark_lost"
                            );
                            "LOST"
                        }
                    }
                };

                emit_status_patch(&afferent_pathway, &status_key, terminal_status).await;

                terminal_status_keys.push_back(status_key.clone());
                if terminal_status_keys.len() > DISPATCH_TERMINAL_RETENTION_LIMIT
                    && let Some(dropped_key) = terminal_status_keys.pop_front()
                {
                    emit_status_drop(&afferent_pathway, dropped_key).await;
                }
            }
        });

        self.dispatch_tx = Some(dispatch_tx);
        self.dispatch_task = Some(dispatch_task);
    }

    async fn shutdown_dispatch_worker(&mut self) {
        self.dispatch_tx.take();
        if let Some(task) = self.dispatch_task.take()
            && let Err(err) = task.await
        {
            tracing::warn!(
                target: "stem.dispatch",
                error = %err,
                "dispatch_worker_join_failed"
            );
        }
    }
}

fn dispatch_status_key(act_instance_id: &str) -> String {
    format!("stem.dispatch.{act_instance_id}.status")
}

async fn emit_status_patch(afferent_pathway: &SenseAfferentPathway, key: &str, value: &str) {
    let mut entries = BTreeMap::new();
    entries.insert(key.to_string(), value.to_string());
    let sense = Sense::NewProprioceptions(ProprioceptionPatch { entries });
    if let Err(err) = afferent_pathway.send(sense).await {
        tracing::warn!(
            target: "stem.dispatch",
            key = key,
            value = value,
            error = %err,
            "failed_to_emit_dispatch_status_patch"
        );
    }
}

async fn emit_status_drop(afferent_pathway: &SenseAfferentPathway, key: String) {
    let sense = Sense::DropProprioceptions(ProprioceptionDropPatch { keys: vec![key.clone()] });
    if let Err(err) = afferent_pathway.send(sense).await {
        tracing::warn!(
            target: "stem.dispatch",
            key = key,
            error = %err,
            "failed_to_emit_dispatch_status_drop"
        );
    }
}

fn merge_neural_signal_descriptor_catalogs(
    cycle_id: u64,
    spine_catalog: NeuralSignalDescriptorCatalog,
    continuity_catalog: NeuralSignalDescriptorCatalog,
    stem_catalog: NeuralSignalDescriptorCatalog,
) -> NeuralSignalDescriptorCatalog {
    let spine_version = spine_catalog.version.clone();
    let continuity_version = continuity_catalog.version.clone();
    let stem_version = stem_catalog.version.clone();

    let mut merged: BTreeMap<NeuralSignalDescriptorRouteKey, NeuralSignalDescriptor> =
        BTreeMap::new();

    for descriptor in spine_catalog.entries {
        merged.insert(
            NeuralSignalDescriptorRouteKey {
                r#type: descriptor.r#type,
                endpoint_id: descriptor.endpoint_id.clone(),
                neural_signal_descriptor_id: descriptor.neural_signal_descriptor_id.clone(),
            },
            descriptor,
        );
    }
    for descriptor in continuity_catalog.entries {
        merged.insert(
            NeuralSignalDescriptorRouteKey {
                r#type: descriptor.r#type,
                endpoint_id: descriptor.endpoint_id.clone(),
                neural_signal_descriptor_id: descriptor.neural_signal_descriptor_id.clone(),
            },
            descriptor,
        );
    }
    for descriptor in stem_catalog.entries {
        merged.insert(
            NeuralSignalDescriptorRouteKey {
                r#type: descriptor.r#type,
                endpoint_id: descriptor.endpoint_id.clone(),
                neural_signal_descriptor_id: descriptor.neural_signal_descriptor_id.clone(),
            },
            descriptor,
        );
    }

    let mut entries = merged.into_values().collect::<Vec<_>>();
    entries.sort_by(|lhs, rhs| {
        lhs.r#type
            .cmp(&rhs.r#type)
            .then_with(|| lhs.endpoint_id.cmp(&rhs.endpoint_id))
            .then_with(|| {
                lhs.neural_signal_descriptor_id
                    .cmp(&rhs.neural_signal_descriptor_id)
            })
    });

    NeuralSignalDescriptorCatalog {
        version: format!(
            "stem:{}:{}:{}:{}",
            cycle_id, spine_version, continuity_version, stem_version
        ),
        entries,
    }
}

fn stem_control_descriptor_catalog() -> NeuralSignalDescriptorCatalog {
    NeuralSignalDescriptorCatalog {
        version: "stem-control:v1".to_string(),
        entries: vec![NeuralSignalDescriptor {
            r#type: NeuralSignalType::Act,
            endpoint_id: "core.control".to_string(),
            neural_signal_descriptor_id: "sleep".to_string(),
            payload_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "seconds": { "type": "integer", "minimum": 1 }
                },
                "required": ["seconds"],
                "additionalProperties": false
            }),
        }],
    }
}
