use std::{collections::VecDeque, sync::Arc};

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::{
    cortex::types::WaitForSenseControlDirective,
    stem::{SenseConsumerHandle, TickGrant},
    types::{PhysicalState, Sense, build_fq_neural_signal_id},
};

mod primary;

pub use primary::{Cortex, CortexTelemetryEvent, CortexTelemetryHook};

#[async_trait]
pub trait PhysicalStateReadPort: Send + Sync {
    async fn snapshot(&self, cycle_id: u64) -> Result<PhysicalState>;
}

pub struct CortexDeps {
    pub tick_grant_rx: mpsc::Receiver<TickGrant>,
    pub afferent_consumer: SenseConsumerHandle,
    pub physical_state_reader: Arc<dyn PhysicalStateReadPort>,
    pub cortex_core: Arc<Cortex>,
}

pub struct CortexRuntime {
    deps: CortexDeps,
    shutdown: CancellationToken,
    cycle_id: u64,
    pending_senses: VecDeque<Sense>,
    ignore_all_triggers_for_ticks_remaining: u64,
    wait_for_sense_gate: Option<WaitForSenseGate>,
}

#[derive(Debug, Clone)]
struct WaitForSenseGate {
    act_instance_id: String,
    expected_fq_sense_ids: Vec<String>,
    ticks_remaining: u64,
}

impl CortexRuntime {
    pub fn new(deps: CortexDeps, shutdown: CancellationToken) -> Self {
        Self {
            cycle_id: 0,
            deps,
            shutdown,
            pending_senses: VecDeque::new(),
            ignore_all_triggers_for_ticks_remaining: 0,
            wait_for_sense_gate: None,
        }
    }

    #[tracing::instrument(name = "cortex_runtime", target = "cortex", skip(self))]
    pub async fn run(mut self) {
        loop {
            tokio::select! {
                biased;
                _ = self.shutdown.cancelled() => {
                    tracing::info!(target = "cortex", "runtime_shutdown");
                    break;
                }
                maybe_tick = self.deps.tick_grant_rx.recv() => {
                    let Some(tick) = maybe_tick else {
                        tracing::info!(target = "cortex", "tick_channel_closed");
                        break;
                    };
                    if let Err(err) = self.on_tick(tick).await {
                        tracing::warn!(target = "cortex", error = %err, "cycle_failed_on_tick");
                    }
                }
                maybe_sense = self.deps.afferent_consumer.recv() => {
                    let Some(sense) = maybe_sense else {
                        tracing::info!(target = "cortex", "afferent_consumer_closed");
                        break;
                    };
                    self.pending_senses.push_back(sense);
                }
            }
        }
    }

    async fn on_tick(&mut self, tick: TickGrant) -> Result<()> {
        self.drain_pending_senses_nonblocking();

        if self.ignore_all_triggers_for_ticks_remaining > 0 {
            self.ignore_all_triggers_for_ticks_remaining = self
                .ignore_all_triggers_for_ticks_remaining
                .saturating_sub(1);
            tracing::debug!(
                target = "cortex",
                tick_seq = tick.tick_seq,
                remaining_ticks = self.ignore_all_triggers_for_ticks_remaining,
                "tick_ignored_by_sleep_gate"
            );
            return Ok(());
        }

        let mut clear_wait_gate = false;
        if let Some(wait_gate) = self.wait_for_sense_gate.as_mut() {
            if wait_for_sense_gate_satisfied(wait_gate, &self.pending_senses) {
                tracing::debug!(
                    target = "cortex",
                    tick_seq = tick.tick_seq,
                    act_instance_id = %wait_gate.act_instance_id,
                    "wait_for_sense_satisfied_on_tick"
                );
                clear_wait_gate = true;
            } else if wait_gate.ticks_remaining > 0 {
                wait_gate.ticks_remaining = wait_gate.ticks_remaining.saturating_sub(1);
                tracing::debug!(
                    target = "cortex",
                    tick_seq = tick.tick_seq,
                    act_instance_id = %wait_gate.act_instance_id,
                    remaining_ticks = wait_gate.ticks_remaining,
                    "tick_skipped_waiting_for_sense"
                );
                return Ok(());
            } else {
                tracing::debug!(
                    target = "cortex",
                    tick_seq = tick.tick_seq,
                    act_instance_id = %wait_gate.act_instance_id,
                    "wait_for_sense_expired_on_tick"
                );
                clear_wait_gate = true;
            }
        }
        if clear_wait_gate {
            self.wait_for_sense_gate = None;
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
            .cortex(&senses, &physical_state)
            .await
            .map_err(|err| anyhow!("cortex_primary_failed: {err}"))?;

        if let Some(ticks) = output.control.ignore_all_trigger_for_ticks {
            self.ignore_all_triggers_for_ticks_remaining = ticks.max(1);
        }

        self.wait_for_sense_gate = output
            .control
            .wait_for_sense
            .and_then(wait_for_sense_gate_from_control);

        Ok(())
    }

    fn drain_pending_senses_nonblocking(&mut self) {
        while let Ok(sense) = self.deps.afferent_consumer.try_recv() {
            self.pending_senses.push_back(sense);
        }
    }
}

fn wait_for_sense_gate_from_control(
    control: WaitForSenseControlDirective,
) -> Option<WaitForSenseGate> {
    let mut expected_fq_sense_ids = control
        .expected_fq_sense_ids
        .into_iter()
        .map(|id| id.trim().to_string())
        .filter(|id| !id.is_empty())
        .collect::<Vec<_>>();
    expected_fq_sense_ids.sort();
    expected_fq_sense_ids.dedup();
    if control.act_instance_id.trim().is_empty()
        || control.wait_ticks == 0
        || expected_fq_sense_ids.is_empty()
    {
        return None;
    }

    Some(WaitForSenseGate {
        act_instance_id: control.act_instance_id,
        expected_fq_sense_ids,
        ticks_remaining: control.wait_ticks,
    })
}

fn wait_for_sense_gate_satisfied(
    wait_gate: &WaitForSenseGate,
    pending_senses: &VecDeque<Sense>,
) -> bool {
    pending_senses.iter().any(|sense| {
        let Some(act_instance_id) = sense.act_instance_id.as_deref() else {
            return false;
        };
        if act_instance_id != wait_gate.act_instance_id {
            return false;
        }
        let fq_sense_id =
            build_fq_neural_signal_id(&sense.endpoint_id, &sense.neural_signal_descriptor_id);
        wait_gate
            .expected_fq_sense_ids
            .iter()
            .any(|expected| expected == &fq_sense_id)
    })
}
