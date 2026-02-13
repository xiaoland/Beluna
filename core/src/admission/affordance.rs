use std::collections::BTreeMap;

use crate::spine::types::CostVector;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DegradationProfile {
    pub profile_id: String,
    pub depth: u8,
    pub capability_loss_score: u16,
    pub cost_multiplier_milli: u16,
    pub capability_id_override: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AffordanceProfile {
    pub profile_id: String,
    pub endpoint_id: String,
    pub capability_id: String,
    pub max_payload_bytes: usize,
    pub base_cost: CostVector,
    pub degradation_profiles: Vec<DegradationProfile>,
}

#[derive(Debug, Clone, Default)]
pub struct AffordanceRegistry {
    profiles_by_key: BTreeMap<(String, String), AffordanceProfile>,
}

impl AffordanceRegistry {
    pub fn new(profiles: impl IntoIterator<Item = AffordanceProfile>) -> Self {
        let mut profiles_by_key = BTreeMap::new();
        for profile in profiles {
            profiles_by_key.insert(
                (profile.endpoint_id.clone(), profile.capability_id.clone()),
                profile,
            );
        }
        Self { profiles_by_key }
    }

    pub fn resolve(&self, endpoint_id: &str, capability_id: &str) -> Option<&AffordanceProfile> {
        self.profiles_by_key
            .get(&(endpoint_id.to_string(), capability_id.to_string()))
    }

    pub fn keys(&self) -> impl Iterator<Item = &(String, String)> {
        self.profiles_by_key.keys()
    }
}

impl From<Vec<AffordanceProfile>> for AffordanceRegistry {
    fn from(value: Vec<AffordanceProfile>) -> Self {
        Self::new(value)
    }
}

impl Default for AffordanceProfile {
    fn default() -> Self {
        Self {
            profile_id: "default-deliberation".to_string(),
            endpoint_id: "core.mind".to_string(),
            capability_id: "deliberate.plan".to_string(),
            max_payload_bytes: 16_384,
            base_cost: CostVector {
                survival_micro: 250,
                time_ms: 120,
                io_units: 1,
                token_units: 128,
            },
            degradation_profiles: vec![
                DegradationProfile {
                    profile_id: "deliberate.plan-lite".to_string(),
                    depth: 1,
                    capability_loss_score: 10,
                    cost_multiplier_milli: 700,
                    capability_id_override: Some("deliberate.plan.lite".to_string()),
                },
                DegradationProfile {
                    profile_id: "deliberate.plan-minimal".to_string(),
                    depth: 2,
                    capability_loss_score: 40,
                    cost_multiplier_milli: 450,
                    capability_id_override: Some("deliberate.plan.minimal".to_string()),
                },
            ],
        }
    }
}
