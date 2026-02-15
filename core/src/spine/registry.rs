use std::{
    collections::BTreeMap,
    sync::{
        Arc, RwLock, Weak,
        atomic::{AtomicU64, Ordering},
    },
};

use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::{
    runtime_types::Act,
    spine::{
        error::{SpineError, backend_failure, registration_invalid, route_not_found},
        ports::{EndpointPort, EndpointRegistryPort},
        types::{
            EndpointCapabilityDescriptor, EndpointExecutionOutcome, RouteKey,
            SpineCapabilityCatalog,
        },
    },
};

struct RegisteredEndpoint {
    endpoint: Arc<dyn EndpointPort>,
    descriptors: BTreeMap<String, EndpointCapabilityDescriptor>,
}

#[derive(Default)]
struct RegistryState {
    version: u64,
    by_endpoint: BTreeMap<String, RegisteredEndpoint>,
    adapter_channels: BTreeMap<u64, mpsc::UnboundedSender<Act>>,
    endpoint_owner_channel: BTreeMap<String, u64>,
}

#[derive(Default)]
pub struct InMemoryEndpointRegistry {
    state: RwLock<RegistryState>,
    next_adapter_channel_seq: AtomicU64,
}

impl InMemoryEndpointRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn version(&self) -> u64 {
        self.state.read().expect("lock poisoned").version
    }

    pub fn allocate_adapter_channel_id(&self, adapter_id: u64) -> u64 {
        let sequence = self
            .next_adapter_channel_seq
            .fetch_add(1, Ordering::Relaxed)
            .saturating_add(1);
        (adapter_id << 32) | (sequence & 0xFFFF_FFFF)
    }

    pub fn attach_adapter_channel(&self, channel_id: u64, tx: mpsc::UnboundedSender<Act>) {
        let mut guard = self.state.write().expect("lock poisoned");
        guard.adapter_channels.insert(channel_id, tx);
    }

    pub fn register_adapter_route(
        self: &Arc<Self>,
        channel_id: u64,
        descriptor: EndpointCapabilityDescriptor,
    ) -> Result<(), SpineError> {
        let route = descriptor.route.clone();
        {
            let guard = self.state.read().expect("lock poisoned");
            if !guard.adapter_channels.contains_key(&channel_id) {
                return Err(backend_failure(format!(
                    "adapter channel {} is not connected",
                    channel_id
                )));
            }

            if let Some(owner) = guard
                .endpoint_owner_channel
                .get(&route.endpoint_id)
                .copied()
                && owner != channel_id
            {
                return Err(backend_failure(format!(
                    "endpoint already owned by adapter channel {}: {}",
                    owner, route.endpoint_id
                )));
            }
        }

        let endpoint: Arc<dyn EndpointPort> = Arc::new(AdapterEndpointPort {
            endpoint_id: route.endpoint_id.clone(),
            registry: Arc::downgrade(self),
        });
        self.register(descriptor, endpoint)?;

        let mut guard = self.state.write().expect("lock poisoned");
        guard
            .endpoint_owner_channel
            .insert(route.endpoint_id.clone(), channel_id);
        Ok(())
    }

    pub fn unregister_adapter_route(
        &self,
        channel_id: u64,
        route: &RouteKey,
    ) -> Option<EndpointCapabilityDescriptor> {
        {
            let guard = self.state.read().expect("lock poisoned");
            if guard
                .endpoint_owner_channel
                .get(&route.endpoint_id)
                .copied()
                != Some(channel_id)
            {
                return None;
            }
        }

        let removed = self.unregister(route);
        self.cleanup_endpoint_owner_channel(&route.endpoint_id, channel_id);
        removed
    }

    pub fn detach_adapter_channel(&self, channel_id: u64) -> Vec<RouteKey> {
        let mut guard = self.state.write().expect("lock poisoned");
        guard.adapter_channels.remove(&channel_id);
        guard
            .endpoint_owner_channel
            .retain(|_, owner| *owner != channel_id);
        Vec::new()
    }

    pub async fn invoke_adapter_endpoint(
        &self,
        endpoint_id: &str,
        act: Act,
    ) -> Result<EndpointExecutionOutcome, SpineError> {
        let tx = {
            let guard = self.state.read().expect("lock poisoned");
            let Some(channel_id) = guard.endpoint_owner_channel.get(endpoint_id).copied() else {
                return Err(route_not_found(format!(
                    "endpoint is not registered: {}",
                    endpoint_id
                )));
            };

            let Some(tx) = guard.adapter_channels.get(&channel_id).cloned() else {
                return Err(backend_failure(format!(
                    "adapter channel {} is unavailable",
                    channel_id
                )));
            };
            tx
        };

        if tx.send(act.clone()).is_err() {
            return Err(backend_failure(format!(
                "failed to dispatch act {} to endpoint {}",
                act.act_id, endpoint_id
            )));
        }

        Ok(EndpointExecutionOutcome::Applied {
            actual_cost_micro: act.requested_resources.survival_micro.max(0),
            reference_id: format!("adapter:act_sent:{}", act.act_id),
        })
    }

    fn cleanup_endpoint_owner_channel(&self, endpoint_id: &str, channel_id: u64) {
        let has_capabilities = {
            let guard = self.state.read().expect("lock poisoned");
            guard.by_endpoint.contains_key(endpoint_id)
        };
        if has_capabilities {
            return;
        }

        let mut guard = self.state.write().expect("lock poisoned");
        if guard.endpoint_owner_channel.get(endpoint_id).copied() == Some(channel_id) {
            guard.endpoint_owner_channel.remove(endpoint_id);
        }
    }
}

impl EndpointRegistryPort for InMemoryEndpointRegistry {
    fn register(
        &self,
        descriptor: EndpointCapabilityDescriptor,
        endpoint: Arc<dyn EndpointPort>,
    ) -> Result<(), SpineError> {
        let route = &descriptor.route;
        if route.endpoint_id.trim().is_empty() || route.capability_id.trim().is_empty() {
            return Err(registration_invalid(
                "route endpoint_id/capability_id cannot be empty",
            ));
        }

        let mut guard = self.state.write().expect("lock poisoned");
        let entry = guard
            .by_endpoint
            .entry(route.endpoint_id.clone())
            .or_insert_with(|| RegisteredEndpoint {
                endpoint: Arc::clone(&endpoint),
                descriptors: BTreeMap::new(),
            });

        entry.endpoint = endpoint;
        entry
            .descriptors
            .insert(route.capability_id.clone(), descriptor);
        guard.version = guard.version.saturating_add(1);
        Ok(())
    }

    fn unregister(&self, route: &RouteKey) -> Option<EndpointCapabilityDescriptor> {
        let mut guard = self.state.write().expect("lock poisoned");
        let mut removed = None;
        let mut remove_endpoint = false;
        if let Some(entry) = guard.by_endpoint.get_mut(&route.endpoint_id) {
            removed = entry.descriptors.remove(&route.capability_id);
            remove_endpoint = entry.descriptors.is_empty();
        }
        if remove_endpoint {
            guard.by_endpoint.remove(&route.endpoint_id);
        }
        if removed.is_some() {
            guard.version = guard.version.saturating_add(1);
        }
        removed
    }

    fn resolve(&self, endpoint_id: &str) -> Option<Arc<dyn EndpointPort>> {
        self.state
            .read()
            .expect("lock poisoned")
            .by_endpoint
            .get(endpoint_id)
            .map(|item| Arc::clone(&item.endpoint))
    }

    fn catalog_snapshot(&self) -> SpineCapabilityCatalog {
        let guard = self.state.read().expect("lock poisoned");
        let mut entries: Vec<_> = guard
            .by_endpoint
            .values()
            .flat_map(|item| item.descriptors.values().cloned())
            .collect();
        entries.sort_by(|lhs, rhs| lhs.route.cmp(&rhs.route));

        SpineCapabilityCatalog {
            version: guard.version,
            entries,
        }
    }
}

struct AdapterEndpointPort {
    endpoint_id: String,
    registry: Weak<InMemoryEndpointRegistry>,
}

#[async_trait]
impl EndpointPort for AdapterEndpointPort {
    async fn invoke(&self, act: Act) -> Result<EndpointExecutionOutcome, SpineError> {
        let Some(registry) = self.registry.upgrade() else {
            return Err(backend_failure("endpoint registry has been dropped"));
        };
        registry
            .invoke_adapter_endpoint(&self.endpoint_id, act)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use crate::{
        runtime_types::Act,
        spine::{
            EndpointCapabilityDescriptor, EndpointExecutionOutcome, RouteKey,
            error::SpineError,
            ports::{EndpointPort, EndpointRegistryPort},
            registry::InMemoryEndpointRegistry,
            types::CostVector,
        },
    };

    #[test]
    fn adapter_channel_id_encodes_adapter_identity() {
        let registry = InMemoryEndpointRegistry::new();

        let a1 = registry.allocate_adapter_channel_id(1);
        let a2 = registry.allocate_adapter_channel_id(2);

        assert_eq!(a1 >> 32, 1);
        assert_eq!(a2 >> 32, 2);
        assert!(a2 > a1);
    }

    struct StubEndpoint;

    #[async_trait]
    impl EndpointPort for StubEndpoint {
        async fn invoke(&self, _act: Act) -> Result<EndpointExecutionOutcome, SpineError> {
            Ok(EndpointExecutionOutcome::Deferred {
                reason_code: "stub".to_string(),
                reference_id: "stub:deferred".to_string(),
            })
        }
    }

    fn descriptor(
        endpoint_id: &str,
        capability_id: &str,
        max_payload_bytes: usize,
    ) -> EndpointCapabilityDescriptor {
        EndpointCapabilityDescriptor {
            route: RouteKey {
                endpoint_id: endpoint_id.to_string(),
                capability_id: capability_id.to_string(),
            },
            payload_schema: serde_json::json!({"type":"object"}),
            max_payload_bytes,
            default_cost: CostVector::default(),
            metadata: Default::default(),
        }
    }

    #[test]
    fn upserts_descriptor_on_duplicate_route_registration() {
        let registry = InMemoryEndpointRegistry::new();
        let endpoint: Arc<dyn EndpointPort> = Arc::new(StubEndpoint);
        let route = RouteKey {
            endpoint_id: "core.mind".to_string(),
            capability_id: "observe.state".to_string(),
        };

        registry
            .register(
                descriptor(&route.endpoint_id, &route.capability_id, 1024),
                Arc::clone(&endpoint),
            )
            .expect("first registration should succeed");
        registry
            .register(
                descriptor(&route.endpoint_id, &route.capability_id, 2048),
                endpoint,
            )
            .expect("second registration should upsert");

        let snapshot = registry.catalog_snapshot();
        assert_eq!(snapshot.entries.len(), 1);
        assert_eq!(snapshot.entries[0].max_payload_bytes, 2048);
    }

    #[test]
    fn catalog_snapshot_is_sorted_and_versioned() {
        let registry = InMemoryEndpointRegistry::new();
        let endpoint: Arc<dyn EndpointPort> = Arc::new(StubEndpoint);

        registry
            .register(
                descriptor("endpoint.z", "cap.2", 1024),
                Arc::clone(&endpoint),
            )
            .expect("registration should succeed");
        registry
            .register(descriptor("endpoint.a", "cap.1", 1024), endpoint)
            .expect("registration should succeed");

        let snapshot = registry.catalog_snapshot();
        assert_eq!(snapshot.version, 2);
        assert_eq!(snapshot.entries.len(), 2);
        assert_eq!(snapshot.entries[0].route.endpoint_id, "endpoint.a");
        assert_eq!(snapshot.entries[1].route.endpoint_id, "endpoint.z");
    }
}
