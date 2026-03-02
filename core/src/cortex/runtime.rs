use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use tokio::sync::mpsc;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;

use crate::{
    cortex::{CognitionState, Cortex, EmittedAct},
    stem::{
        ActProducerHandle, AfferentRuleControlPort, DeferralRuleOverwriteInput,
        EfferentActEnvelope, SenseConsumerHandle, TickGrant,
    },
    types::{NeuralSignalType, PhysicalState, Sense, build_fq_neural_signal_id},
};

#[async_trait]
pub trait PhysicalStateReadPort: Send + Sync {
    async fn snapshot(&self, cycle_id: u64) -> Result<PhysicalState>;
}

pub struct CortexDeps {
    pub tick_grant_rx: mpsc::Receiver<TickGrant>,
    pub afferent_consumer: SenseConsumerHandle,
    pub afferent_rule_control: Arc<dyn AfferentRuleControlPort>,
    pub efferent_producer: ActProducerHandle,
    pub init_cognition_state: CognitionState,
    pub physical_state_reader: Arc<dyn PhysicalStateReadPort>,
    pub cortex_core: Arc<Cortex>,
}

pub struct CortexRuntime {
    deps: CortexDeps,
    shutdown: CancellationToken,
    cycle_id: u64,
    cognition_state: CognitionState,
    pending_senses: VecDeque<Sense>,
    ignore_all_trigger_until: Option<Instant>,
}

impl CortexRuntime {
    pub fn new(deps: CortexDeps, shutdown: CancellationToken) -> Self {
        Self {
            cycle_id: 0,
            cognition_state: deps.init_cognition_state.clone(),
            deps,
            shutdown,
            pending_senses: VecDeque::new(),
            ignore_all_trigger_until: None,
        }
    }

    #[tracing::instrument(name = "cortex_runtime", target = "cortex", skip(self))]
    pub async fn run(mut self) {
        loop {
            tokio::select! {
                _ = self.shutdown.cancelled() => {
                    tracing::info!(target = "cortex", "runtime_shutdown");
                    break;
                }
                maybe_tick = self.deps.tick_grant_rx.recv() => {
                    let Some(_tick) = maybe_tick else {
                        tracing::info!(target = "cortex", "tick_channel_closed");
                        break;
                    };
                    if let Err(err) = self.try_execute_cycle(true).await {
                        tracing::warn!(target = "cortex", error = %err, "cycle_failed_on_tick");
                    }
                }
                maybe_sense = self.deps.afferent_consumer.recv() => {
                    let Some(sense) = maybe_sense else {
                        tracing::info!(target = "cortex", "afferent_consumer_closed");
                        break;
                    };
                    self.pending_senses.push_back(sense);
                    if let Err(err) = self.try_execute_cycle(false).await {
                        tracing::warn!(target = "cortex", error = %err, "cycle_failed_on_sense");
                    }
                }
            }
        }
    }

    async fn try_execute_cycle(&mut self, tick_triggered: bool) -> Result<()> {
        if let Some(deadline) = self.ignore_all_trigger_until
            && Instant::now() < deadline
        {
            tracing::debug!(target = "cortex", "trigger_ignored_by_sleep_gate");
            return Ok(());
        }
        self.ignore_all_trigger_until = None;

        self.drain_pending_senses_nonblocking();
        if !tick_triggered && self.pending_senses.is_empty() {
            return Ok(());
        }

        let senses = self.pending_senses.drain(..).collect::<Vec<_>>();

        self.cycle_id = self.cycle_id.saturating_add(1);
        let physical_state = self
            .deps
            .physical_state_reader
            .snapshot(self.cycle_id)
            .await
            .map_err(|err| anyhow!("physical_state_snapshot_failed: {err}"))?;

        let output = self
            .deps
            .cortex_core
            .cortex(&senses, &physical_state, &self.cognition_state)
            .await
            .map_err(|err| anyhow!("cortex_primary_failed: {err}"))?;

        if let Err(err) = self
            .deps
            .cortex_core
            .persist_cognition_state(output.new_cognition_state.clone())
            .await
        {
            tracing::warn!(target = "cortex", error = %err, "persist_cognition_state_failed");
        }
        self.cognition_state = output.new_cognition_state;

        for (idx, emitted) in output.emitted_acts.into_iter().enumerate() {
            let act_seq_no = (idx as u64).saturating_add(1);
            let act_instance_id = emitted.act.act_instance_id.clone();
            if let Err(err) = self
                .deps
                .efferent_producer
                .enqueue(EfferentActEnvelope {
                    cycle_id: self.cycle_id,
                    act_seq_no,
                    act: emitted.act.clone(),
                })
                .await
            {
                tracing::warn!(
                    target = "cortex",
                    cycle_id = self.cycle_id,
                    act_seq_no = act_seq_no,
                    act_instance_id = %act_instance_id,
                    error = %err,
                    "efferent_enqueue_failed"
                );
            }
            self.wait_for_sense_if_requested(&physical_state, &emitted, &act_instance_id)
                .await;
        }

        if let Some(seconds) = output.control.ignore_all_trigger_for_seconds {
            self.ignore_all_trigger_until =
                Some(Instant::now() + Duration::from_secs(seconds.max(1)));
        }

        Ok(())
    }

    async fn wait_for_sense_if_requested(
        &mut self,
        physical_state: &PhysicalState,
        emitted: &EmittedAct,
        act_instance_id: &str,
    ) {
        if emitted.wait_for_sense_seconds == 0 {
            return;
        }

        let expected_fq_sense_ids = emitted
            .expected_fq_sense_ids
            .iter()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();

        self.apply_wait_gate_rule(physical_state, &expected_fq_sense_ids)
            .await;

        let wait_result = timeout(
            Duration::from_secs(emitted.wait_for_sense_seconds),
            self.wait_for_matching_sense(act_instance_id, &expected_fq_sense_ids),
        )
        .await;
        if wait_result.is_err() {
            tracing::warn!(
                target = "cortex",
                cycle_id = self.cycle_id,
                act_instance_id = act_instance_id,
                wait_for_sense_seconds = emitted.wait_for_sense_seconds,
                "wait_for_sense_timeout"
            );
        }

        self.clear_wait_gate_rule().await;
    }

    async fn wait_for_matching_sense(
        &mut self,
        act_instance_id: &str,
        expected_fq_sense_ids: &std::collections::BTreeSet<String>,
    ) -> Option<Sense> {
        loop {
            let maybe_sense = self.deps.afferent_consumer.recv().await;
            let sense = maybe_sense?;

            if sense.act_instance_id.as_deref() == Some(act_instance_id) {
                return Some(sense);
            }

            let fq_sense_id =
                build_fq_neural_signal_id(&sense.endpoint_id, &sense.neural_signal_descriptor_id);
            if expected_fq_sense_ids.contains(&fq_sense_id) {
                return Some(sense);
            }

            self.pending_senses.push_back(sense);
        }
    }

    async fn apply_wait_gate_rule(
        &self,
        physical_state: &PhysicalState,
        expected_fq_sense_ids: &std::collections::BTreeSet<String>,
    ) {
        if expected_fq_sense_ids.is_empty() {
            return;
        }

        let non_target_fq_ids = physical_state
            .ns_descriptor
            .entries
            .iter()
            .filter(|descriptor| descriptor.r#type == NeuralSignalType::Sense)
            .map(|descriptor| {
                build_fq_neural_signal_id(
                    &descriptor.endpoint_id,
                    &descriptor.neural_signal_descriptor_id,
                )
            })
            .filter(|fq_sense_id| !expected_fq_sense_ids.contains(fq_sense_id))
            .collect::<Vec<_>>();

        if non_target_fq_ids.is_empty() {
            return;
        }

        let pattern = non_target_fq_ids
            .iter()
            .map(|item| regex::escape(item))
            .collect::<Vec<_>>()
            .join("|");

        let result = self
            .deps
            .afferent_rule_control
            .overwrite_rule(DeferralRuleOverwriteInput {
                rule_id: "__cortex_wait_non_target__".to_string(),
                min_weight: None,
                fq_sense_id_pattern: Some(format!("^(?:{pattern})$")),
            })
            .await;
        if let Err(err) = result {
            tracing::warn!(
                target = "cortex",
                error = %err,
                "wait_gate_rule_overwrite_failed"
            );
        }
    }

    async fn clear_wait_gate_rule(&self) {
        let result = self
            .deps
            .afferent_rule_control
            .overwrite_rule(DeferralRuleOverwriteInput {
                rule_id: "__cortex_wait_non_target__".to_string(),
                min_weight: None,
                fq_sense_id_pattern: Some("^$".to_string()),
            })
            .await;
        if let Err(err) = result {
            tracing::warn!(
                target = "cortex",
                error = %err,
                "wait_gate_rule_clear_failed"
            );
        }
    }

    fn drain_pending_senses_nonblocking(&mut self) {
        while let Ok(sense) = self.deps.afferent_consumer.try_recv() {
            self.pending_senses.push_back(sense);
        }
    }

    pub fn set_ignore_all_trigger_until(&mut self, deadline: Option<Instant>) {
        self.ignore_all_trigger_until = deadline;
    }
}
