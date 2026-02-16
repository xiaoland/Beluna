use serde::{Deserialize, Serialize};

pub type SenseId = String;
pub type ActId = String;

pub fn is_uuid_v4(id: &str) -> bool {
    uuid::Uuid::parse_str(id)
        .map(|uuid| uuid.get_version_num() == 4)
        .unwrap_or(false)
}

pub fn is_uuid_v7(id: &str) -> bool {
    uuid::Uuid::parse_str(id)
        .map(|uuid| uuid.get_version_num() == 7)
        .unwrap_or(false)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RequestedResources {
    pub survival_micro: i64,
    pub time_ms: u64,
    pub io_units: u64,
    pub token_units: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SenseDatum {
    pub sense_id: SenseId,
    pub source: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CapabilityPatch {
    #[serde(default)]
    pub entries: Vec<crate::spine::types::EndpointCapabilityDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CapabilityDropPatch {
    #[serde(default)]
    pub routes: Vec<crate::spine::types::RouteKey>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Sense {
    Domain(SenseDatum),
    Sleep,
    NewCapabilities(CapabilityPatch),
    DropCapabilities(CapabilityDropPatch),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Act {
    pub act_id: ActId,
    #[serde(default)]
    pub based_on: Vec<SenseId>,
    pub body_endpoint_name: String,
    pub capability_id: String,
    pub capability_instance_id: String,
    pub normalized_payload: serde_json::Value,
    #[serde(default)]
    pub requested_resources: RequestedResources,
}
