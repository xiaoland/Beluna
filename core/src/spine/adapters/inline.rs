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
use uuid::Uuid;

use crate::{
    afferent_pathway::SenseAfferentPathway,
    config::InlineAdapterConfig,
    spine::{
        ActDispatchResult, Endpoint, EndpointBinding, EndpointCapabilityDescriptor, runtime::Spine,
    },
    types::{Act, CapabilityDropPatch, CapabilityPatch, Sense, SenseDatum},
};

struct RegisteredInlineEndpoint {
    body_endpoint_id: Uuid,
    act_tx: mpsc::Sender<Arc<Act>>,
    sense_task: JoinHandle<()>,
}

pub struct InlineEndpointRuntimeHandles {
    pub act_rx: mpsc::Receiver<Arc<Act>>,
    pub sense_tx: mpsc::Sender<Arc<SenseDatum>>,
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
        capabilities: Vec<EndpointCapabilityDescriptor>,
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

        let registered_entries = match spine.add_capabilities(handle.body_endpoint_id, capabilities)
        {
            Ok(entries) => entries,
            Err(err) => {
                spine.remove_endpoint(handle.body_endpoint_id);
                return Err(anyhow!(err.to_string()));
            }
        };

        let (act_tx, act_rx) = mpsc::channel::<Arc<Act>>(self.act_queue_capacity);
        let (sense_tx, mut sense_rx) = mpsc::channel::<Arc<SenseDatum>>(self.sense_queue_capacity);

        let adapter = Arc::downgrade(self);
        let shutdown = self.shutdown.clone();
        let endpoint_name_for_task = endpoint_name.clone();
        let body_endpoint_id = handle.body_endpoint_id;
        let sense_task = tokio::spawn(async move {
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
                        if let Err(err) = adapter
                            .afferent_pathway
                            .send(Sense::Domain((*sense).clone()))
                            .await
                        {
                            eprintln!(
                                "inline adapter dropped sense from endpoint '{}': {}",
                                endpoint_name_for_task,
                                err
                            );
                        }
                    }
                }
            }

            if let Some(adapter) = adapter.upgrade() {
                adapter
                    .remove_endpoint_by_name(&endpoint_name_for_task, body_endpoint_id, false)
                    .await;
            }
        });

        {
            let mut state = self.endpoints.lock().await;
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
                .send(Sense::NewCapabilities(CapabilityPatch {
                    entries: registered_entries,
                }))
                .await
        {
            eprintln!("inline adapter dropped capability patch after attach: {err}");
        }

        Ok(InlineEndpointRuntimeHandles { act_rx, sense_tx })
    }

    async fn enqueue_act(&self, endpoint_name: &str, act: Act) -> Result<ActDispatchResult> {
        let act_id = act.act_id.clone();

        let tx = {
            let state = self.endpoints.lock().await;
            state.get(endpoint_name).map(|entry| entry.act_tx.clone())
        };
        let Some(tx) = tx else {
            return Ok(ActDispatchResult::Rejected {
                reason_code: "endpoint_not_found".to_string(),
                reference_id: format!("inline_adapter:endpoint_not_found:{act_id}"),
            });
        };

        if tx.send(Arc::new(act)).await.is_err() {
            self.remove_endpoint_by_name(endpoint_name, Uuid::nil(), true)
                .await;
            return Ok(ActDispatchResult::Rejected {
                reason_code: "endpoint_unavailable".to_string(),
                reference_id: format!("inline_adapter:endpoint_unavailable:{act_id}"),
            });
        }

        Ok(ActDispatchResult::Acknowledged {
            reference_id: format!("inline_adapter:enqueued:{act_id}"),
        })
    }

    async fn remove_endpoint_by_name(
        &self,
        endpoint_name: &str,
        expected_id: Uuid,
        abort_task: bool,
    ) {
        let removed = {
            let mut state = self.endpoints.lock().await;
            let should_remove = state
                .get(endpoint_name)
                .map(|entry| expected_id.is_nil() || entry.body_endpoint_id == expected_id)
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
            let routes = spine.remove_endpoint(removed.body_endpoint_id);
            if !routes.is_empty()
                && let Err(err) = self
                    .afferent_pathway
                    .send(Sense::DropCapabilities(CapabilityDropPatch { routes }))
                    .await
            {
                eprintln!("inline adapter dropped capability drop during detach: {err}");
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        afferent_pathway::SenseAfferentPathway,
        config::{InlineAdapterConfig, SpineRuntimeConfig},
        spine::{RouteKey, runtime::Spine, types::CostVector},
        types::{RequestedResources, Sense},
    };

    fn descriptor(capability_id: &str) -> EndpointCapabilityDescriptor {
        EndpointCapabilityDescriptor {
            route: RouteKey {
                endpoint_id: "placeholder".to_string(),
                capability_id: capability_id.to_string(),
            },
            payload_schema: serde_json::json!({"type":"object"}),
            max_payload_bytes: 1024,
            default_cost: CostVector {
                survival_micro: 1,
                time_ms: 1,
                io_units: 1,
                token_units: 1,
            },
            metadata: Default::default(),
        }
    }

    #[tokio::test]
    async fn attach_registers_endpoint_and_forwards_sense() {
        let (afferent_pathway, mut sense_rx) = SenseAfferentPathway::new(8);
        let spine = Spine::new(
            &SpineRuntimeConfig { adapters: vec![] },
            afferent_pathway.clone(),
        );
        let adapter = Arc::new(SpineInlineAdapter::new(
            1,
            InlineAdapterConfig::default(),
            spine.clone(),
            afferent_pathway,
            CancellationToken::new(),
        ));

        let mut handles = adapter
            .attach_inline_endpoint("std-shell".to_string(), vec![descriptor("tool.shell.exec")])
            .await
            .expect("attach should succeed");

        let _ = sense_rx.recv().await;

        let act = Act {
            act_id: uuid::Uuid::now_v7().to_string(),
            based_on: vec![],
            body_endpoint_name: "std-shell".to_string(),
            capability_id: "tool.shell.exec".to_string(),
            capability_instance_id: "shell.instance".to_string(),
            normalized_payload: serde_json::json!({"argv":["echo","ok"]}),
            requested_resources: RequestedResources::default(),
        };

        let dispatch = spine
            .dispatch_act(act.clone())
            .await
            .expect("dispatch should succeed");
        assert!(matches!(dispatch, ActDispatchResult::Acknowledged { .. }));

        let dispatched = handles.act_rx.recv().await.expect("endpoint receives act");
        assert_eq!(dispatched.act_id, act.act_id);

        handles
            .sense_tx
            .send(Arc::new(SenseDatum {
                sense_id: uuid::Uuid::new_v4().to_string(),
                source: "body.inline.test".to_string(),
                payload: serde_json::json!({"act_id": act.act_id}),
            }))
            .await
            .expect("sense send should work");

        let forwarded = sense_rx.recv().await.expect("sense should be forwarded");
        match forwarded {
            Sense::Domain(payload) => assert_eq!(payload.source, "body.inline.test"),
            _ => panic!("expected domain sense"),
        }
    }
}
