use std::collections::{BTreeMap, BTreeSet};

use crate::{
    admission::types::RequestedResources,
    cortex::{AffordanceCapability, CapabilityCatalog},
    spine::types::SpineCapabilityCatalog,
};

pub fn to_cortex_catalog(source: &SpineCapabilityCatalog) -> CapabilityCatalog {
    let mut grouped: BTreeMap<String, Vec<&crate::spine::types::EndpointCapabilityDescriptor>> =
        BTreeMap::new();

    for entry in &source.entries {
        grouped
            .entry(entry.route.affordance_key.clone())
            .or_default()
            .push(entry);
    }

    let mut affordances = Vec::with_capacity(grouped.len());
    for (affordance_key, entries) in grouped {
        let representative = entries
            .first()
            .expect("grouped affordance entries should never be empty");

        let mut handles = BTreeSet::new();
        for entry in &entries {
            handles.insert(entry.route.capability_handle.clone());
        }

        affordances.push(AffordanceCapability {
            affordance_key,
            allowed_capability_handles: handles.into_iter().collect(),
            payload_schema: representative.payload_schema.clone(),
            max_payload_bytes: representative.max_payload_bytes,
            default_resources: RequestedResources {
                survival_micro: representative.default_cost.survival_micro,
                time_ms: representative.default_cost.time_ms,
                io_units: representative.default_cost.io_units,
                token_units: representative.default_cost.token_units,
            },
        });
    }

    CapabilityCatalog {
        version: format!("spine:v{}", source.version),
        affordances,
    }
}

#[cfg(test)]
mod tests {
    use crate::spine::types::{
        CostVector, EndpointCapabilityDescriptor, RouteKey, SpineCapabilityCatalog,
    };

    use super::to_cortex_catalog;

    #[test]
    fn groups_entries_by_affordance_and_dedupes_handles() {
        let source = SpineCapabilityCatalog {
            version: 7,
            entries: vec![
                EndpointCapabilityDescriptor {
                    route: RouteKey {
                        affordance_key: "deliberate.plan".to_string(),
                        capability_handle: "cap.core".to_string(),
                    },
                    payload_schema: serde_json::json!({"type":"object"}),
                    max_payload_bytes: 1024,
                    default_cost: CostVector::default(),
                    metadata: Default::default(),
                },
                EndpointCapabilityDescriptor {
                    route: RouteKey {
                        affordance_key: "deliberate.plan".to_string(),
                        capability_handle: "cap.core.lite".to_string(),
                    },
                    payload_schema: serde_json::json!({"type":"object"}),
                    max_payload_bytes: 1024,
                    default_cost: CostVector::default(),
                    metadata: Default::default(),
                },
            ],
        };

        let target = to_cortex_catalog(&source);
        assert_eq!(target.version, "spine:v7");
        assert_eq!(target.affordances.len(), 1);
        assert_eq!(target.affordances[0].allowed_capability_handles.len(), 2);
    }
}
