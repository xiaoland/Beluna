use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum OwnerScope {
    MainRuntime,
    StemTick,
    StemAfferentPathway,
    StemProprioception,
    StemDescriptorCatalog,
    StemEfferentPathway,
    CortexPrimary,
    CortexAttention,
    CortexCleanup,
    CortexSenseHelper,
    CortexGoalForest,
    CortexActsHelper,
    AiGatewayTransport,
    AiGatewayChat,
    SpineEndpoint { segment: String },
    SpineAdapter { segment: String },
}

impl OwnerScope {
    pub(crate) fn spine_endpoint(endpoint_id: &str) -> Self {
        Self::SpineEndpoint {
            segment: canonical_scope_segment(endpoint_id),
        }
    }

    pub(crate) fn spine_adapter(adapter_name: &str) -> Self {
        Self::SpineAdapter {
            segment: canonical_scope_segment(adapter_name),
        }
    }

    pub(crate) fn name(&self) -> Cow<'static, str> {
        match self {
            OwnerScope::MainRuntime => Cow::Borrowed("beluna.core.main.runtime"),
            OwnerScope::StemTick => Cow::Borrowed("beluna.core.stem.tick"),
            OwnerScope::StemAfferentPathway => Cow::Borrowed("beluna.core.stem.afferent-pathway"),
            OwnerScope::StemProprioception => Cow::Borrowed("beluna.core.stem.proprioception"),
            OwnerScope::StemDescriptorCatalog => {
                Cow::Borrowed("beluna.core.stem.descriptor-catalog")
            }
            OwnerScope::StemEfferentPathway => Cow::Borrowed("beluna.core.stem.efferent-pathway"),
            OwnerScope::CortexPrimary => Cow::Borrowed("beluna.core.cortex.primary"),
            OwnerScope::CortexAttention => Cow::Borrowed("beluna.core.cortex.attention"),
            OwnerScope::CortexCleanup => Cow::Borrowed("beluna.core.cortex.cleanup"),
            OwnerScope::CortexSenseHelper => Cow::Borrowed("beluna.core.cortex.sense-helper"),
            OwnerScope::CortexGoalForest => Cow::Borrowed("beluna.core.cortex.goal-forest"),
            OwnerScope::CortexActsHelper => Cow::Borrowed("beluna.core.cortex.acts-helper"),
            OwnerScope::AiGatewayTransport => Cow::Borrowed("beluna.core.ai-gateway.transport"),
            OwnerScope::AiGatewayChat => Cow::Borrowed("beluna.core.ai-gateway.chat"),
            OwnerScope::SpineEndpoint { segment } => {
                Cow::Owned(format!("beluna.core.spine.endpoint.{segment}"))
            }
            OwnerScope::SpineAdapter { segment } => {
                Cow::Owned(format!("beluna.core.spine.adapter.{segment}"))
            }
        }
    }
}

pub(crate) fn canonical_scope_segment(raw: &str) -> String {
    if raw.is_empty() {
        return "unknown".to_string();
    }

    let mut segment = String::with_capacity(raw.len());
    for byte in raw.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' => {
                segment.push(*byte as char);
            }
            _ => {
                segment.push_str(&format!("%{byte:02X}"));
            }
        }
    }
    segment
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrganResponseStatus {
    Ok,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DispatchOutcomeClass {
    Acknowledged,
    Rejected,
    Lost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DescriptorCatalogChangeMode {
    Snapshot,
    Update,
    Drop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterLifecycleState {
    Enabled,
    Disabled,
    Faulted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndpointLifecycleTransition {
    Connected,
    Disconnected,
    Registered,
    Dropped,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalizes_dynamic_spine_owner_segments_without_collisions() {
        assert_eq!(
            OwnerScope::spine_endpoint("endpoint.alpha/1").name(),
            "beluna.core.spine.endpoint.endpoint%2Ealpha%2F1"
        );
        assert_eq!(
            OwnerScope::spine_adapter("unix socket").name(),
            "beluna.core.spine.adapter.unix%20socket"
        );
        assert_eq!(
            OwnerScope::spine_endpoint("endpoint%2Ealpha").name(),
            "beluna.core.spine.endpoint.endpoint%252Ealpha"
        );
    }
}
