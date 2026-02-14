use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use crate::spine::{
    error::{SpineError, registration_invalid, route_conflict},
    ports::{EndpointPort, EndpointRegistryPort},
    types::{EndpointRegistration, RouteKey, SpineCapabilityCatalog},
};

struct RegisteredEndpoint {
    registration: EndpointRegistration,
    endpoint: Arc<dyn EndpointPort>,
}

#[derive(Default)]
struct RegistryState {
    version: u64,
    by_route: BTreeMap<RouteKey, RegisteredEndpoint>,
}

#[derive(Default)]
pub struct InMemoryEndpointRegistry {
    state: RwLock<RegistryState>,
}

impl InMemoryEndpointRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn version(&self) -> u64 {
        self.state.read().expect("lock poisoned").version
    }
}

impl EndpointRegistryPort for InMemoryEndpointRegistry {
    fn register(
        &self,
        registration: EndpointRegistration,
        endpoint: Arc<dyn EndpointPort>,
    ) -> Result<(), SpineError> {
        if registration.endpoint_id.trim().is_empty() {
            return Err(registration_invalid("endpoint_id cannot be empty"));
        }

        let route = &registration.descriptor.route;
        if route.endpoint_id.trim().is_empty() || route.capability_id.trim().is_empty() {
            return Err(registration_invalid(
                "route endpoint_id/capability_id cannot be empty",
            ));
        }

        let mut guard = self.state.write().expect("lock poisoned");
        if guard.by_route.contains_key(route) {
            return Err(route_conflict(format!(
                "route already registered: {}::{}",
                route.endpoint_id, route.capability_id
            )));
        }

        for existing in guard
            .by_route
            .values()
            .filter(|item| item.registration.descriptor.route.endpoint_id == route.endpoint_id)
        {
            let lhs = &existing.registration.descriptor;
            let rhs = &registration.descriptor;
            if lhs.payload_schema != rhs.payload_schema
                || lhs.max_payload_bytes != rhs.max_payload_bytes
                || lhs.default_cost != rhs.default_cost
            {
                return Err(registration_invalid(format!(
                    "inconsistent descriptor for endpoint '{}'",
                    route.endpoint_id
                )));
            }
        }

        guard.by_route.insert(
            route.clone(),
            RegisteredEndpoint {
                registration,
                endpoint,
            },
        );
        guard.version = guard.version.saturating_add(1);
        Ok(())
    }

    fn unregister(&self, route: &RouteKey) -> Option<EndpointRegistration> {
        let mut guard = self.state.write().expect("lock poisoned");
        let removed = guard.by_route.remove(route);
        if removed.is_some() {
            guard.version = guard.version.saturating_add(1);
        }
        removed.map(|item| item.registration)
    }

    fn resolve(&self, route: &RouteKey) -> Option<Arc<dyn EndpointPort>> {
        self.state
            .read()
            .expect("lock poisoned")
            .by_route
            .get(route)
            .map(|item| Arc::clone(&item.endpoint))
    }

    fn catalog_snapshot(&self) -> SpineCapabilityCatalog {
        let guard = self.state.read().expect("lock poisoned");
        let mut entries: Vec<_> = guard
            .by_route
            .values()
            .map(|item| item.registration.descriptor.clone())
            .collect();
        entries.sort_by(|lhs, rhs| lhs.route.cmp(&rhs.route));

        SpineCapabilityCatalog {
            version: guard.version,
            entries,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use crate::spine::{
        EndpointCapabilityDescriptor, EndpointExecutionOutcome, EndpointInvocation,
        EndpointRegistration, RouteKey,
        error::SpineError,
        ports::{EndpointPort, EndpointRegistryPort},
        registry::InMemoryEndpointRegistry,
        types::CostVector,
    };

    struct StubEndpoint;

    #[async_trait]
    impl EndpointPort for StubEndpoint {
        async fn invoke(
            &self,
            _invocation: EndpointInvocation,
        ) -> Result<EndpointExecutionOutcome, SpineError> {
            Ok(EndpointExecutionOutcome::Deferred {
                reason_code: "stub".to_string(),
                reference_id: "stub:deferred".to_string(),
            })
        }
    }

    fn registration(
        endpoint_id: &str,
        capability_id: &str,
        max_payload_bytes: usize,
    ) -> EndpointRegistration {
        EndpointRegistration {
            endpoint_id: format!("ep:{}:{}", endpoint_id, capability_id),
            descriptor: EndpointCapabilityDescriptor {
                route: RouteKey {
                    endpoint_id: endpoint_id.to_string(),
                    capability_id: capability_id.to_string(),
                },
                payload_schema: serde_json::json!({"type":"object"}),
                max_payload_bytes,
                default_cost: CostVector::default(),
                metadata: Default::default(),
            },
        }
    }

    #[test]
    fn rejects_duplicate_route_registration() {
        let registry = InMemoryEndpointRegistry::new();
        let endpoint: Arc<dyn EndpointPort> = Arc::new(StubEndpoint);
        let route = RouteKey {
            endpoint_id: "core.mind".to_string(),
            capability_id: "observe.state".to_string(),
        };

        registry
            .register(
                registration(&route.endpoint_id, &route.capability_id, 1024),
                Arc::clone(&endpoint),
            )
            .expect("first registration should succeed");

        let err = registry
            .register(
                registration(&route.endpoint_id, &route.capability_id, 1024),
                endpoint,
            )
            .expect_err("duplicate route should fail");

        assert!(matches!(
            err.kind,
            crate::spine::error::SpineErrorKind::RouteConflict
        ));
    }

    #[test]
    fn rejects_inconsistent_descriptor_for_same_affordance() {
        let registry = InMemoryEndpointRegistry::new();
        let endpoint: Arc<dyn EndpointPort> = Arc::new(StubEndpoint);

        registry
            .register(
                registration("core.mind", "observe.state", 1024),
                Arc::clone(&endpoint),
            )
            .expect("first registration should succeed");

        let err = registry
            .register(
                registration("core.mind", "observe.state.remote", 2048),
                endpoint,
            )
            .expect_err("inconsistent endpoint descriptor should fail");

        assert!(matches!(
            err.kind,
            crate::spine::error::SpineErrorKind::RegistrationInvalid
        ));
    }

    #[test]
    fn catalog_snapshot_is_sorted_and_versioned() {
        let registry = InMemoryEndpointRegistry::new();
        let endpoint: Arc<dyn EndpointPort> = Arc::new(StubEndpoint);

        registry
            .register(
                registration("endpoint.z", "cap.2", 1024),
                Arc::clone(&endpoint),
            )
            .expect("registration should succeed");
        registry
            .register(registration("endpoint.a", "cap.1", 1024), endpoint)
            .expect("registration should succeed");

        let snapshot = registry.catalog_snapshot();
        assert_eq!(snapshot.version, 2);
        assert_eq!(snapshot.entries.len(), 2);
        assert_eq!(snapshot.entries[0].route.endpoint_id, "endpoint.a");
        assert_eq!(snapshot.entries[1].route.endpoint_id, "endpoint.z");
    }
}
