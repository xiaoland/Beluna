use std::{
    collections::BTreeMap,
    sync::{Arc, Weak},
};

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use tokio::{
    sync::{Mutex, mpsc},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use tracing::Instrument;

use crate::{
    afferent_pathway::SenseAfferentPathway,
    config::InlineAdapterConfig,
    spine::{ActDispatchResult, Endpoint, EndpointBinding, NeuralSignalDescriptor, runtime::Spine},
    types::{Act, NeuralSignalDescriptorDropPatch, NeuralSignalDescriptorPatch, Sense, SenseDatum},
};

#[derive(Debug, Clone)]
pub struct InlineSenseDatum {
    pub sense_id: String,
    pub neural_signal_descriptor_id: String,
    pub payload: serde_json::Value,
}

struct RegisteredInlineEndpoint {
    body_endpoint_id: String,
    act_tx: mpsc::Sender<Arc<Act>>,
    sense_task: JoinHandle<()>,
}

pub struct InlineEndpointRuntimeHandles {
    pub act_rx: mpsc::Receiver<Arc<Act>>,
    pub sense_tx: mpsc::Sender<Arc<InlineSenseDatum>>,
}

pub struct SpineInlineAdapter {
    adapter_id: u64,
    spine: Weak<Spine>,
    afferent_pathway: SenseAfferentPathway,
    shutdown: CancellationToken,
    act_queue_capacity: usize,
    sense_queue_capacity: usize,
    endpoints: Mutex<BTreeMap<String, RegisteredInlineEndpoint>>,
}

impl SpineInlineAdapter {
    pub fn new(
        adapter_id: u64,
        config: InlineAdapterConfig,
        spine: Arc<Spine>,
        afferent_pathway: SenseAfferentPathway,
        shutdown: CancellationToken,
    ) -> Self {
        Self {
            adapter_id,
            spine: Arc::downgrade(&spine),
            afferent_pathway,
            shutdown,
            act_queue_capacity: config.act_queue_capacity.max(1),
            sense_queue_capacity: config.sense_queue_capacity.max(1),
            endpoints: Mutex::new(BTreeMap::new()),
        }
    }

    pub fn adapter_id(&self) -> u64 {
        self.adapter_id
    }

    pub async fn attach_inline_endpoint(
        self: &Arc<Self>,
        endpoint_name: String,
        capabilities: Vec<NeuralSignalDescriptor>,
    ) -> Result<InlineEndpointRuntimeHandles> {
        let endpoint_name = endpoint_name.trim().to_string();
        if endpoint_name.is_empty() {
            return Err(anyhow!("endpoint_name cannot be empty"));
        }

        let endpoint_proxy: Arc<dyn Endpoint> = Arc::new(InlineAdapterEndpointProxy {
            endpoint_name: endpoint_name.clone(),
            adapter: Arc::downgrade(self),
        });

        let spine = self
            .spine
            .upgrade()
            .ok_or_else(|| anyhow!("spine runtime is unavailable"))?;

        let handle = spine
            .add_endpoint(
                &endpoint_name,
                EndpointBinding::Inline(endpoint_proxy),
                Vec::new(),
            )
            .map_err(|err| anyhow!(err.to_string()))?;

        let body_endpoint_id = handle.body_endpoint_id.clone();
        let registered_entries = match spine.add_capabilities(&body_endpoint_id, capabilities) {
            Ok(entries) => entries,
            Err(err) => {
                spine.remove_endpoint(&body_endpoint_id);
                return Err(anyhow!(err.to_string()));
            }
        };

        let (act_tx, act_rx) = mpsc::channel::<Arc<Act>>(self.act_queue_capacity);
        let (sense_tx, mut sense_rx) =
            mpsc::channel::<Arc<InlineSenseDatum>>(self.sense_queue_capacity);

        let adapter = Arc::downgrade(self);
        let shutdown = self.shutdown.clone();
        let endpoint_name_for_task = endpoint_name.clone();
        let body_endpoint_id_for_task = body_endpoint_id.clone();
        let sense_span = tracing::info_span!(
            target: "spine.inline_adapter",
            "inline_sense_task",
            endpoint_name = %endpoint_name_for_task,
            body_endpoint_id = %body_endpoint_id_for_task
        );
        let sense_task = tokio::spawn(
            async move {
                loop {
                    tokio::select! {
                        _ = shutdown.cancelled() => {
                            break;
                        }
                        maybe_sense = sense_rx.recv() => {
                            let Some(sense) = maybe_sense else {
                                break;
                            };
                            let Some(adapter) = adapter.upgrade() else {
                                break;
                            };
                            // Inline endpoints do not carry endpoint_id; adapter injects the bound endpoint id.
                            let sense = SenseDatum {
                                sense_id: sense.sense_id.clone(),
                                endpoint_id: body_endpoint_id_for_task.clone(),
                                neural_signal_descriptor_id: sense.neural_signal_descriptor_id.clone(),
                                payload: sense.payload.clone(),
                            };
                            if let Err(err) = adapter.afferent_pathway.send(Sense::Domain(sense)).await {
                                tracing::warn!(
                                    target: "spine.inline_adapter",
                                    endpoint_name = endpoint_name_for_task,
                                    error = %err,
                                    "dropped_sense_from_inline_endpoint"
                                );
                            }
                        }
                    }
                }

                if let Some(adapter) = adapter.upgrade() {
                    adapter
                        .remove_endpoint_by_name(&endpoint_name_for_task, Some(&body_endpoint_id_for_task), false)
                        .await;
                }
            }
            .instrument(sense_span),
        );

        {
            let mut state = self.endpoints.lock().await;
            if state.contains_key(&endpoint_name) {
                return Err(anyhow!(
                    "inline endpoint '{}' is already attached",
                    endpoint_name
                ));
            }
            state.insert(
                endpoint_name,
                RegisteredInlineEndpoint {
                    body_endpoint_id,
                    act_tx,
                    sense_task,
                },
            );
        }

        if !registered_entries.is_empty()
            && let Err(err) = self
                .afferent_pathway
                .send(Sense::NewNeuralSignalDescriptors(
                    NeuralSignalDescriptorPatch {
                        entries: registered_entries,
                    },
                ))
                .await
        {
            tracing::warn!(
                target: "spine.inline_adapter",
                error = %err,
                "dropped_neural_signal_descriptor_patch_after_attach"
            );
        }

        Ok(InlineEndpointRuntimeHandles { act_rx, sense_tx })
    }

    async fn enqueue_act(&self, endpoint_name: &str, act: Act) -> Result<ActDispatchResult> {
        let act_id = act.act_id.clone();
        tracing::debug!(
            target: "spine.inline_adapter",
            endpoint_id = endpoint_name,
            act_id = %act_id,
            neural_signal_descriptor_id = %act.neural_signal_descriptor_id,
            "enqueue_act_for_inline_endpoint"
        );

        let tx = {
            let state = self.endpoints.lock().await;
            state.get(endpoint_name).map(|entry| entry.act_tx.clone())
        };
        let Some(tx) = tx else {
            tracing::warn!(
                target: "spine.inline_adapter",
                endpoint_id = endpoint_name,
                act_id = %act_id,
                "inline_endpoint_not_found_for_dispatch"
            );
            return Ok(ActDispatchResult::Rejected {
                reason_code: "endpoint_not_found".to_string(),
                reference_id: format!("inline_adapter:endpoint_not_found:{act_id}"),
            });
        };

        if tx.send(Arc::new(act)).await.is_err() {
            tracing::warn!(
                target: "spine.inline_adapter",
                endpoint_id = endpoint_name,
                act_id = %act_id,
                "inline_endpoint_unavailable_during_dispatch"
            );
            self.remove_endpoint_by_name(endpoint_name, None, true)
                .await;
            return Ok(ActDispatchResult::Rejected {
                reason_code: "endpoint_unavailable".to_string(),
                reference_id: format!("inline_adapter:endpoint_unavailable:{act_id}"),
            });
        }

        tracing::debug!(
            target: "spine.inline_adapter",
            endpoint_id = endpoint_name,
            act_id = %act_id,
            "act_enqueued_for_inline_endpoint"
        );
        Ok(ActDispatchResult::Acknowledged {
            reference_id: format!("inline_adapter:enqueued:{act_id}"),
        })
    }

    async fn remove_endpoint_by_name(
        &self,
        endpoint_name: &str,
        expected_id: Option<&str>,
        abort_task: bool,
    ) {
        let removed = {
            let mut state = self.endpoints.lock().await;
            let should_remove = state
                .get(endpoint_name)
                .map(|entry| {
                    expected_id
                        .map(|id| entry.body_endpoint_id == id)
                        .unwrap_or(true)
                })
                .unwrap_or(false);
            if should_remove {
                state.remove(endpoint_name)
            } else {
                None
            }
        };

        let Some(removed) = removed else {
            return;
        };

        if abort_task {
            removed.sense_task.abort();
        }

        if let Some(spine) = self.spine.upgrade() {
            let routes = spine.remove_endpoint(&removed.body_endpoint_id);
            if !routes.is_empty()
                && let Err(err) = self
                    .afferent_pathway
                    .send(Sense::DropNeuralSignalDescriptors(
                        NeuralSignalDescriptorDropPatch { routes },
                    ))
                    .await
            {
                tracing::warn!(
                    target: "spine.inline_adapter",
                    error = %err,
                    "dropped_neural_signal_descriptor_drop_during_detach"
                );
            }
        }
    }
}

struct InlineAdapterEndpointProxy {
    endpoint_name: String,
    adapter: Weak<SpineInlineAdapter>,
}

#[async_trait]
impl Endpoint for InlineAdapterEndpointProxy {
    async fn invoke(&self, act: Act) -> Result<ActDispatchResult, crate::spine::SpineError> {
        let Some(adapter) = self.adapter.upgrade() else {
            tracing::warn!(
                target: "spine.inline_adapter",
                endpoint_id = %self.endpoint_name,
                act_id = %act.act_id,
                "inline_adapter_unavailable_for_dispatch"
            );
            return Ok(ActDispatchResult::Rejected {
                reason_code: "adapter_unavailable".to_string(),
                reference_id: format!("inline_adapter:unavailable:{}", act.act_id),
            });
        };

        adapter
            .enqueue_act(&self.endpoint_name, act)
            .await
            .map_err(|err| crate::spine::error::internal_error(err.to_string()))
    }
}
