use std::{
    collections::BTreeMap,
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use tokio::{
    sync::{RwLock, mpsc},
    time::Instant,
};
use tokio_util::sync::CancellationToken;

use crate::{
    types::{
        NeuralSignalDescriptor, NeuralSignalDescriptorCatalog, NeuralSignalDescriptorDropPatch,
        NeuralSignalDescriptorPatch, NeuralSignalDescriptorRouteKey, PhysicalLedgerSnapshot,
        PhysicalState, ProprioceptionDropPatch, ProprioceptionPatch,
    },
};

#[derive(Clone)]
pub struct StemPhysicalStateStore {
    inner: Arc<RwLock<PhysicalState>>,
}

impl StemPhysicalStateStore {
    pub fn new(startup_proprioception: BTreeMap<String, String>) -> Self {
        let state = PhysicalState {
            cycle_id: 0,
            ledger: PhysicalLedgerSnapshot::default(),
            ns_descriptor: NeuralSignalDescriptorCatalog {
                version: "stem:v0".to_string(),
                entries: Vec::new(),
            },
            proprioception: startup_proprioception,
        };
        Self {
            inner: Arc::new(RwLock::new(state)),
        }
    }

    pub fn shared_state(&self) -> Arc<RwLock<PhysicalState>> {
        Arc::clone(&self.inner)
    }

    pub async fn ns_descriptor_snapshot(&self) -> NeuralSignalDescriptorCatalog {
        self.inner.read().await.ns_descriptor.clone()
    }

    pub async fn snapshot_for_cycle(&self, cycle_id: u64) -> PhysicalState {
        let mut state = self.inner.read().await.clone();
        state.cycle_id = cycle_id;
        state
    }

    pub async fn proprioception_snapshot(&self) -> BTreeMap<String, String> {
        self.inner.read().await.proprioception.clone()
    }

    async fn apply_neural_signal_descriptor_patch_inner(&self, patch: NeuralSignalDescriptorPatch) {
        if patch.entries.is_empty() {
            return;
        }
        let mut state = self.inner.write().await;
        let entries = &mut state.ns_descriptor.entries;
        for descriptor in patch.entries {
            let route = route_key_from_descriptor(&descriptor);
            if let Some(existing) = entries
                .iter_mut()
                .find(|item| route_key_from_descriptor(item) == route)
            {
                *existing = descriptor;
            } else {
                entries.push(descriptor);
            }
        }
        sort_ns_descriptor_entries(entries);
        state.ns_descriptor.version = next_stem_ns_descriptor_version(&state.ns_descriptor.version);
    }

    async fn apply_neural_signal_descriptor_drop_inner(
        &self,
        patch: NeuralSignalDescriptorDropPatch,
    ) {
        if patch.routes.is_empty() {
            return;
        }
        let mut state = self.inner.write().await;
        let routes = patch.routes.into_iter().collect::<std::collections::BTreeSet<_>>();
        let original_len = state.ns_descriptor.entries.len();
        state
            .ns_descriptor
            .entries
            .retain(|descriptor| !routes.contains(&route_key_from_descriptor(descriptor)));
        let changed = state.ns_descriptor.entries.len() != original_len;
        if changed {
            state.ns_descriptor.version =
                next_stem_ns_descriptor_version(&state.ns_descriptor.version);
        }
    }

    async fn apply_proprioception_patch_inner(&self, patch: ProprioceptionPatch) {
        if patch.entries.is_empty() {
            return;
        }
        let mut state = self.inner.write().await;
        for (key, value) in patch.entries {
            state.proprioception.insert(key, value);
        }
    }

    async fn apply_proprioception_drop_inner(&self, patch: ProprioceptionDropPatch) {
        if patch.keys.is_empty() {
            return;
        }
        let mut state = self.inner.write().await;
        for key in patch.keys {
            state.proprioception.remove(&key);
        }
    }
}

fn route_key_from_descriptor(
    descriptor: &NeuralSignalDescriptor,
) -> NeuralSignalDescriptorRouteKey {
    NeuralSignalDescriptorRouteKey {
        r#type: descriptor.r#type,
        endpoint_id: descriptor.endpoint_id.clone(),
        neural_signal_descriptor_id: descriptor.neural_signal_descriptor_id.clone(),
    }
}

fn sort_ns_descriptor_entries(entries: &mut [NeuralSignalDescriptor]) {
    entries.sort_by(|lhs, rhs| {
        lhs.r#type
            .cmp(&rhs.r#type)
            .then_with(|| lhs.endpoint_id.cmp(&rhs.endpoint_id))
            .then_with(|| {
                lhs.neural_signal_descriptor_id
                    .cmp(&rhs.neural_signal_descriptor_id)
            })
    });
}

fn next_stem_ns_descriptor_version(current: &str) -> String {
    let next = current
        .strip_prefix("stem:v")
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0)
        .saturating_add(1);
    format!("stem:v{next}")
}

#[async_trait]
pub trait StemControlPort: Send + Sync {
    async fn apply_neural_signal_descriptor_patch(&self, patch: NeuralSignalDescriptorPatch);
    async fn apply_neural_signal_descriptor_drop(&self, patch: NeuralSignalDescriptorDropPatch);
    async fn apply_proprioception_patch(&self, patch: ProprioceptionPatch);
    async fn apply_proprioception_drop(&self, patch: ProprioceptionDropPatch);
}

#[async_trait]
impl StemControlPort for StemPhysicalStateStore {
    async fn apply_neural_signal_descriptor_patch(&self, patch: NeuralSignalDescriptorPatch) {
        self.apply_neural_signal_descriptor_patch_inner(patch).await;
    }

    async fn apply_neural_signal_descriptor_drop(&self, patch: NeuralSignalDescriptorDropPatch) {
        self.apply_neural_signal_descriptor_drop_inner(patch).await;
    }

    async fn apply_proprioception_patch(&self, patch: ProprioceptionPatch) {
        self.apply_proprioception_patch_inner(patch).await;
    }

    async fn apply_proprioception_drop(&self, patch: ProprioceptionDropPatch) {
        self.apply_proprioception_drop_inner(patch).await;
    }
}

#[derive(Debug, Clone)]
pub struct TickGrant {
    pub tick_seq: u64,
    pub emitted_at: Instant,
}

#[derive(Debug)]
pub struct StemDeps {
    pub tick_interval_ms: u64,
    pub tick_grant_tx: mpsc::Sender<TickGrant>,
}

pub struct StemTickRuntime {
    deps: StemDeps,
    shutdown: CancellationToken,
}

impl StemTickRuntime {
    pub fn new(deps: StemDeps, shutdown: CancellationToken) -> Self {
        Self { deps, shutdown }
    }

    #[tracing::instrument(name = "stem_tick_runtime", target = "stem", skip(self))]
    pub async fn run(self) {
        let mut tick =
            tokio::time::interval(Duration::from_millis(self.deps.tick_interval_ms.max(1)));
        let mut tick_seq = 0_u64;
        loop {
            tokio::select! {
                _ = self.shutdown.cancelled() => {
                    tracing::info!(target = "stem", "tick_runtime_shutdown");
                    break;
                }
                _ = tick.tick() => {
                    tick_seq = tick_seq.saturating_add(1);
                    if self.deps.tick_grant_tx.send(TickGrant {
                        tick_seq,
                        emitted_at: Instant::now(),
                    }).await.is_err() {
                        tracing::info!(
                            target = "stem",
                            tick_seq = tick_seq,
                            "tick_runtime_receiver_closed"
                        );
                        break;
                    }
                }
            }
        }
    }
}
