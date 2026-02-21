use std::{collections::BTreeMap, sync::Arc, time::Duration};

use anyhow::Result;
use tokio::{
    sync::{Mutex, mpsc},
    time::{Instant, MissedTickBehavior},
};

use crate::{
    config::TickMissedBehavior,
    continuity::{ContinuityEngine, DispatchContext as ContinuityDispatchContext},
    cortex::Cortex,
    spine::Spine,
    types::{
        Act, DispatchDecision, NeuralSignalDescriptor, NeuralSignalDescriptorCatalog,
        NeuralSignalDescriptorRouteKey, NeuralSignalType, PhysicalLedgerSnapshot, PhysicalState,
        Sense,
    },
};

enum StemMode {
    Active,
    SleepingUntil(Instant),
}

pub struct Stem {
    cycle_id: u64,
    cortex: Arc<Cortex>,
    continuity: Arc<Mutex<ContinuityEngine>>,
    spine: Arc<Spine>,
    sense_rx: mpsc::Receiver<Sense>,
    tick_interval_ms: u64,
    tick_missed_behavior: TickMissedBehavior,
}

impl Stem {
    pub fn new(
        cortex: Arc<Cortex>,
        continuity: Arc<Mutex<ContinuityEngine>>,
        spine: Arc<Spine>,
        sense_rx: mpsc::Receiver<Sense>,
        tick_interval_ms: u64,
        tick_missed_behavior: TickMissedBehavior,
    ) -> Self {
        Self {
            cycle_id: 0,
            cortex,
            continuity,
            spine,
            sense_rx,
            tick_interval_ms: tick_interval_ms.max(1),
            tick_missed_behavior,
        }
    }

    #[tracing::instrument(name = "stem_run", target = "stem", skip(self))]
    pub async fn run(mut self) -> Result<()> {
        let mut mode = StemMode::Active;
        let mut tick = tokio::time::interval(Duration::from_millis(self.tick_interval_ms));
        tick.set_missed_tick_behavior(match self.tick_missed_behavior {
            TickMissedBehavior::Skip => MissedTickBehavior::Skip,
        });

        loop {
            match mode {
                StemMode::Active => {
                    tick.tick().await;

                    let senses = self.drain_senses_nonblocking();
                    if senses.iter().any(|sense| matches!(sense, Sense::Hibernate)) {
                        break;
                    }

                    if let Some(deadline) = self.execute_cycle(senses).await? {
                        mode = StemMode::SleepingUntil(deadline);
                    }
                }
                StemMode::SleepingUntil(deadline) => {
                    let now = Instant::now();
                    if now >= deadline {
                        if let Some(next_deadline) = self.execute_cycle(Vec::new()).await? {
                            mode = StemMode::SleepingUntil(next_deadline);
                        } else {
                            mode = StemMode::Active;
                        }
                        continue;
                    }

                    let wait = deadline.saturating_duration_since(now);
                    tokio::select! {
                        _ = tokio::time::sleep(wait) => {
                            if let Some(next_deadline) = self.execute_cycle(Vec::new()).await? {
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

                            if let Some(next_deadline) = self.execute_cycle(senses).await? {
                                mode = StemMode::SleepingUntil(next_deadline);
                            } else {
                                mode = StemMode::Active;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn execute_cycle(&mut self, sense_batch: Vec<Sense>) -> Result<Option<Instant>> {
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
                return Ok(None);
            }
        };

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
                    act_id = %act.act_id,
                    "sleep_act_intercepted"
                );
                break;
            }

            let ctx = ContinuityDispatchContext {
                cycle_id: self.cycle_id,
                act_seq_no: seq_no,
            };
            let continuity_decision = self.continuity.lock().await.on_act(&act, &ctx)?;
            if matches!(continuity_decision, DispatchDecision::Break) {
                tracing::info!(
                    target: "stem.dispatch",
                    cycle_id = self.cycle_id,
                    seq_no = seq_no,
                    reason = "continuity_break",
                    "dispatch_break"
                );
                continue;
            }

            let spine_decision = self.spine.on_act(act.clone()).await?;
            if matches!(spine_decision, DispatchDecision::Break) {
                tracing::info!(
                    target: "stem.dispatch",
                    cycle_id = self.cycle_id,
                    seq_no = seq_no,
                    reason = "spine_break",
                    "dispatch_break"
                );
            }
        }

        Ok(sleep_deadline)
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
        })
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
