use std::collections::HashMap;

use crate::ai_gateway::{
    error::{GatewayError, invalid_request},
    types::{AIGatewayConfig, BackendId, BackendProfile, DEFAULT_ROUTE_ALIAS, ModelTarget},
    types_chat::CanonicalRequest,
};

#[derive(Clone)]
pub struct BackendRouter {
    backends: HashMap<BackendId, BackendProfile>,
    route_aliases: HashMap<String, ModelTarget>,
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
            if profile.models.is_empty() {
                return Err(invalid_request(format!(
                    "backend '{}' must define at least one model",
                    profile.id
                )));
            }

            let mut model_ids = std::collections::HashSet::new();
            for model in &profile.models {
                if model.id.trim().is_empty() {
                    return Err(invalid_request(format!(
                        "backend '{}' contains empty model id",
                        profile.id
                    )));
                }
                if !model_ids.insert(model.id.clone()) {
                    return Err(invalid_request(format!(
                        "backend '{}' has duplicate model id '{}'",
                        profile.id, model.id
                    )));
                }
            }

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

        let route_aliases = config
            .route_aliases
            .iter()
            .map(|(alias, target)| (alias.trim().to_string(), target.clone()))
            .collect::<HashMap<_, _>>();

        if !route_aliases.contains_key(DEFAULT_ROUTE_ALIAS) {
            return Err(invalid_request(format!(
                "ai_gateway.route_aliases must include '{}'",
                DEFAULT_ROUTE_ALIAS
            )));
        }

        for (alias, target) in &route_aliases {
            if alias.is_empty() {
                return Err(invalid_request(
                    "ai_gateway.route_aliases cannot contain empty keys",
                ));
            }

            let backend = backends.get(&target.backend_id).ok_or_else(|| {
                invalid_request(format!(
                    "alias '{}' points to missing backend '{}'",
                    alias, target.backend_id
                ))
            })?;

            if !backend
                .models
                .iter()
                .any(|model| model.id == target.model_id)
            {
                return Err(invalid_request(format!(
                    "alias '{}' points to unknown model '{}' on backend '{}'",
                    alias, target.model_id, target.backend_id
                )));
            }
        }

        Ok(Self {
            backends,
            route_aliases,
        })
    }

    pub fn select(&self, req: &CanonicalRequest) -> Result<SelectedBackend, GatewayError> {
        let route_ref = req
            .route_hint
            .clone()
            .unwrap_or_else(|| DEFAULT_ROUTE_ALIAS.to_string());

        let target = self.resolve_route_ref(&route_ref)?;
        let profile = self
            .backends
            .get(&target.backend_id)
            .cloned()
            .ok_or_else(|| {
                invalid_request(format!(
                    "selected backend '{}' does not exist",
                    target.backend_id
                ))
            })?;

        if !profile
            .models
            .iter()
            .any(|model| model.id == target.model_id)
        {
            return Err(invalid_request(format!(
                "selected model '{}' does not exist on backend '{}'",
                target.model_id, target.backend_id
            )));
        }

        Ok(SelectedBackend {
            backend_id: target.backend_id,
            profile,
            resolved_model: target.model_id,
        })
    }

    fn resolve_route_ref(&self, route_ref: &str) -> Result<ModelTarget, GatewayError> {
        let trimmed = route_ref.trim();
        if trimmed.is_empty() {
            return Err(invalid_request("route reference cannot be empty"));
        }

        if let Some((backend_id, model_id)) = trimmed.split_once('/') {
            if backend_id.trim().is_empty() || model_id.trim().is_empty() {
                return Err(invalid_request(format!(
                    "invalid route '{}', expected '<backend-id>/<model-id>'",
                    route_ref
                )));
            }
            return Ok(ModelTarget {
                backend_id: backend_id.trim().to_string(),
                model_id: model_id.trim().to_string(),
            });
        }

        self.route_aliases.get(trimmed).cloned().ok_or_else(|| {
            invalid_request(format!(
                "unknown route alias '{}'; expected alias or '<backend-id>/<model-id>'",
                trimmed
            ))
        })
    }
}
