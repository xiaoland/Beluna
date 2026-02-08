use std::collections::HashMap;

use crate::ai_gateway::{
    error::{GatewayError, invalid_request},
    types::{AIGatewayConfig, BackendId, BackendProfile, CanonicalRequest},
};

#[derive(Clone)]
pub struct BackendRouter {
    default_backend: BackendId,
    backends: HashMap<BackendId, BackendProfile>,
}

#[derive(Debug, Clone)]
pub struct SelectedBackend {
    pub backend_id: BackendId,
    pub profile: BackendProfile,
    pub resolved_model: String,
}

impl BackendRouter {
    pub fn new(config: &AIGatewayConfig) -> Result<Self, GatewayError> {
        if config.backends.is_empty() {
            return Err(invalid_request("ai_gateway.backends must not be empty"));
        }

        let mut backends = HashMap::new();
        for profile in &config.backends {
            if backends
                .insert(profile.id.clone(), profile.clone())
                .is_some()
            {
                return Err(invalid_request(format!(
                    "duplicate backend id '{}' in ai_gateway.backends",
                    profile.id
                )));
            }
        }

        if !backends.contains_key(&config.default_backend) {
            return Err(invalid_request(format!(
                "ai_gateway.default_backend '{}' does not exist",
                config.default_backend
            )));
        }

        Ok(Self {
            default_backend: config.default_backend.clone(),
            backends,
        })
    }

    pub fn select(&self, req: &CanonicalRequest) -> Result<SelectedBackend, GatewayError> {
        let backend_id = req
            .backend_hint
            .clone()
            .unwrap_or_else(|| self.default_backend.clone());

        let profile = self.backends.get(&backend_id).ok_or_else(|| {
            invalid_request(format!(
                "selected backend '{}' does not exist (no fallback in MVP)",
                backend_id
            ))
        })?;

        let resolved_model = req
            .model_override
            .clone()
            .unwrap_or_else(|| profile.default_model.clone());

        Ok(SelectedBackend {
            backend_id,
            profile: profile.clone(),
            resolved_model,
        })
    }
}
