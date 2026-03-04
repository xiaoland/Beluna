use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::{
    stem::{SenseConsumerHandle, TickGrant},
    types::PhysicalState,
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
    pending_senses: VecDeque<crate::types::Sense>,
    ignore_all_trigger_until: Option<Instant>,
    pending_primary_continuation: bool,
}

impl CortexRuntime {
    pub fn new(deps: CortexDeps, shutdown: CancellationToken) -> Self {
        Self {
            cycle_id: 0,
            deps,
            shutdown,
            pending_senses: VecDeque::new(),
            ignore_all_trigger_until: None,
            pending_primary_continuation: false,
        }
    }

    #[tracing::instrument(name = "cortex_runtime", target = "cortex", skip(self))]
    pub async fn run(mut self) {
        loop {
            if self.pending_primary_continuation {
                if let Err(err) = self.try_execute_cycle(false).await {
                    tracing::warn!(
                        target = "cortex",
                        error = %err,
                        "cycle_failed_on_primary_continuation"
                    );
                    self.pending_primary_continuation = false;
                }
                continue;
            }

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
        if !self.pending_primary_continuation
            && let Some(deadline) = self.ignore_all_trigger_until
            && Instant::now() < deadline
        {
            tracing::debug!(target = "cortex", "trigger_ignored_by_sleep_gate");
            return Ok(());
        }
        if !self.pending_primary_continuation {
            self.ignore_all_trigger_until = None;
            self.drain_pending_senses_nonblocking();
            if !tick_triggered && self.pending_senses.is_empty() {
                return Ok(());
            }
        }

        let senses = if self.pending_primary_continuation {
            Vec::new()
        } else {
            self.pending_senses.drain(..).collect::<Vec<_>>()
        };

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

        self.pending_primary_continuation = output.pending_primary_continuation;

        if let Some(seconds) = output.control.ignore_all_trigger_for_seconds {
            self.ignore_all_trigger_until =
                Some(Instant::now() + Duration::from_secs(seconds.max(1)));
        }

        Ok(())
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
