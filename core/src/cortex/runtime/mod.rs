use std::{collections::VecDeque, sync::Arc};

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::{
    stem::{SenseConsumerHandle, TickGrant},
    types::{PhysicalState, Sense},
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
}

impl CortexRuntime {
    pub fn new(deps: CortexDeps, shutdown: CancellationToken) -> Self {
        Self {
            cycle_id: 0,
            deps,
            shutdown,
            pending_senses: VecDeque::new(),
            ignore_all_triggers_for_ticks_remaining: 0,
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

        Ok(())
    }

    fn drain_pending_senses_nonblocking(&mut self) {
        while let Ok(sense) = self.deps.afferent_consumer.try_recv() {
            self.pending_senses.push_back(sense);
        }
    }
}
