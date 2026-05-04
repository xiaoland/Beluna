use std::{collections::BTreeMap, sync::Arc};

use anyhow::{Result, anyhow};
use tokio::{
    sync::{Mutex, mpsc},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use tracing::Instrument;

use crate::{
    observability::{contract::AdapterLifecycleState, runtime as observability_runtime},
    spine::{
        AdapterContext, AdapterId, NeuralSignalDescriptor, SpineAdapterPort,
        types::ActDispatchResult,
    },
    types::{Act, Sense},
};

pub mod config;
pub use config::InlineAdapterConfig;

#[derive(Debug, Clone)]
pub struct InlineSenseDatum {
    pub sense_instance_id: String,
    pub neural_signal_descriptor_id: String,
    pub payload: String,
    pub weight: f64,
    pub act_instance_id: Option<String>,
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
    adapter_id: AdapterId,
    port: Arc<dyn SpineAdapterPort>,
    sense_tx: mpsc::UnboundedSender<Sense>,
    shutdown: CancellationToken,
    act_queue_capacity: usize,
    sense_queue_capacity: usize,
    endpoints: Mutex<BTreeMap<String, RegisteredInlineEndpoint>>,
}

impl SpineInlineAdapter {
    pub fn from_config(
        config: InlineAdapterConfig,
        context: AdapterContext,
    ) -> (Arc<Self>, JoinHandle<Result<()>>) {
        let adapter = Arc::new(Self::new(config, &context));
        let dispatch_task = Self::spawn_dispatch_task(Arc::clone(&adapter), context.act_rx);
        (adapter, dispatch_task)
    }

    fn new(config: InlineAdapterConfig, context: &AdapterContext) -> Self {
        Self {
            adapter_id: context.adapter_id,
            port: Arc::clone(&context.port),
            sense_tx: context.sense_tx.clone(),
            shutdown: context.shutdown.clone(),
            act_queue_capacity: config.act_queue_capacity.max(1),
            sense_queue_capacity: config.sense_queue_capacity.max(1),
            endpoints: Mutex::new(BTreeMap::new()),
        }
    }

    fn spawn_dispatch_task(
        adapter: Arc<Self>,
        mut act_rx: mpsc::UnboundedReceiver<Act>,
    ) -> JoinHandle<Result<()>> {
        let shutdown = adapter.shutdown.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown.cancelled() => {
                        break;
                    }
                    maybe_act = act_rx.recv() => {
                        let Some(act) = maybe_act else {
                            break;
                        };
                        adapter.enqueue_act(&act.endpoint_id.clone(), act).await?;
                    }
                }
            }
            Ok(())
        })
    }

    pub fn adapter_id(&self) -> u64 {
        self.adapter_id
    }

    pub fn emit_started(&self) {
        tracing::info!(
            target: "spine",
            adapter_type = "inline",
            adapter_id = self.adapter_id,
            act_queue_capacity = self.act_queue_capacity,
            sense_queue_capacity = self.sense_queue_capacity,
            "adapter_started"
        );
        observability_runtime::emit_spine_adapter_lifecycle(
            "inline",
            &self.adapter_id.to_string(),
            AdapterLifecycleState::Enabled,
            None,
        );
    }

    pub async fn attach_inline_endpoint(
        self: &Arc<Self>,
        endpoint_name: String,
        ns_descriptors: Vec<NeuralSignalDescriptor>,
    ) -> Result<InlineEndpointRuntimeHandles> {
        let endpoint_name = endpoint_name.trim().to_string();
        if endpoint_name.is_empty() {
            return Err(anyhow!("endpoint_name cannot be empty"));
        }

        let handle = self
            .port
            .register_endpoint(self.adapter_id, &endpoint_name)
            .await?;

        let body_endpoint_id = handle.body_endpoint_id.clone();
        let descriptor_count = ns_descriptors.len();
        let accepted_descriptors = self
            .port
            .add_ns_descriptors(&body_endpoint_id, ns_descriptors)
            .await;
        let accepted_descriptors = match accepted_descriptors {
            Ok(accepted_descriptors) => accepted_descriptors,
            Err(err) => {
                self.port.drop_endpoint(&body_endpoint_id).await;
                return Err(anyhow!(err.to_string()));
            }
        };
        if accepted_descriptors.len() != descriptor_count {
            self.port.drop_endpoint(&body_endpoint_id).await;
            return Err(anyhow!(
                "inline endpoint '{}' descriptor registration incomplete: expected {}, accepted {}",
                endpoint_name,
                descriptor_count,
                accepted_descriptors.len()
            ));
        }

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
                            let sense = Sense {
                                sense_instance_id: sense.sense_instance_id.clone(),
                                endpoint_id: body_endpoint_id_for_task.clone(),
                                neural_signal_descriptor_id: sense.neural_signal_descriptor_id.clone(),
                                payload: sense.payload.clone(),
                                weight: sense.weight.clamp(0.0, 1.0),
                                act_instance_id: sense.act_instance_id.clone(),
                            };
                            if adapter.sense_tx.send(sense).is_err() {
                                break;
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

        self.port.publish_topology_proprioception_snapshot().await;

        Ok(InlineEndpointRuntimeHandles { act_rx, sense_tx })
    }

    async fn enqueue_act(&self, body_endpoint_id: &str, act: Act) -> Result<ActDispatchResult> {
        let act_instance_id = act.act_instance_id.clone();
        tracing::debug!(
            target: "spine.inline_adapter",
            endpoint_id = body_endpoint_id,
            act_instance_id = %act_instance_id,
            neural_signal_descriptor_id = %act.neural_signal_descriptor_id,
            "enqueue_act_for_inline_endpoint"
        );

        let target = {
            let state = self.endpoints.lock().await;
            state
                .iter()
                .find(|(_, entry)| entry.body_endpoint_id == body_endpoint_id)
                .map(|(endpoint_name, entry)| (endpoint_name.clone(), entry.act_tx.clone()))
        };
        let Some((endpoint_name, tx)) = target else {
            tracing::warn!(
                target: "spine.inline_adapter",
                endpoint_id = body_endpoint_id,
                act_instance_id = %act_instance_id,
                "inline_endpoint_not_found_for_dispatch"
            );
            self.port.drop_endpoint(body_endpoint_id).await;
            self.port.publish_topology_proprioception_snapshot().await;
            return Ok(ActDispatchResult::Rejected {
                reason_code: "endpoint_not_found".to_string(),
                reference_id: format!("inline_adapter:endpoint_not_found:{act_instance_id}"),
            });
        };

        if tx.send(Arc::new(act)).await.is_err() {
            tracing::warn!(
                target: "spine.inline_adapter",
                endpoint_id = endpoint_name,
                act_instance_id = %act_instance_id,
                "inline_endpoint_unavailable_during_dispatch"
            );
            self.remove_endpoint_by_name(&endpoint_name, None, true)
                .await;
            return Ok(ActDispatchResult::Rejected {
                reason_code: "endpoint_unavailable".to_string(),
                reference_id: format!("inline_adapter:endpoint_unavailable:{act_instance_id}"),
            });
        }

        tracing::debug!(
            target: "spine.inline_adapter",
            endpoint_id = endpoint_name,
            act_instance_id = %act_instance_id,
            "act_enqueued_for_inline_endpoint"
        );
        Ok(ActDispatchResult::Acknowledged {
            reference_id: format!("inline_adapter:enqueued:{act_instance_id}"),
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

        self.port.drop_endpoint(&removed.body_endpoint_id).await;
        self.port.publish_topology_proprioception_snapshot().await;
    }
}
