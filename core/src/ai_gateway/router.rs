use std::collections::HashMap;

use crate::ai_gateway::{
    error::{GatewayError, invalid_request},
    types::{
        AIGatewayConfig, BackendId, BackendProfile, CHAT_CAPABILITY_ID, ChatRouteAlias,
        ChatRouteKey, ChatRouteRef, DEFAULT_ROUTE_ALIAS, ModelTarget,
    },
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
        let mut route_aliases = HashMap::new();

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

                // Build route aliases from model-level aliases
                for alias in &model.aliases {
                    let trimmed_alias = alias.trim();
                    if trimmed_alias.is_empty() {
                        return Err(invalid_request(format!(
                            "backend '{}' model '{}' contains empty alias",
                            profile.id, model.id
                        )));
                    }
                    if route_aliases.contains_key(trimmed_alias) {
                        return Err(invalid_request(format!(
                            "duplicate alias '{}' in ai_gateway.backends",
                            trimmed_alias
                        )));
                    }
                    route_aliases.insert(
                        trimmed_alias.to_string(),
                        ModelTarget {
                            backend_id: profile.id.clone(),
                            model_id: model.id.clone(),
                        },
                    );
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

        if !route_aliases.contains_key(DEFAULT_ROUTE_ALIAS) {
            return Err(invalid_request(format!(
                "ai_gateway.backends must define alias '{}' on some model",
                DEFAULT_ROUTE_ALIAS
            )));
        }

        Ok(Self {
            backends,
            route_aliases,
        })
    }

    pub fn select_route_ref(
        &self,
        route_ref: Option<&ChatRouteRef>,
    ) -> Result<SelectedBackend, GatewayError> {
        let target = match route_ref {
            Some(ChatRouteRef::Alias(alias)) => self.resolve_chat_alias(alias)?,
            Some(ChatRouteRef::Key(key)) => self.resolve_chat_key(key)?,
            None => self.resolve_chat_alias(&ChatRouteAlias::default_chat())?,
        };
        self.selected_from_target(target)
    }

    fn selected_from_target(&self, target: ModelTarget) -> Result<SelectedBackend, GatewayError> {
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

    fn resolve_chat_alias(&self, alias: &ChatRouteAlias) -> Result<ModelTarget, GatewayError> {
        if alias.capability != CHAT_CAPABILITY_ID {
            return Err(invalid_request(format!(
                "unsupported capability '{}' in route alias; expected '{}'",
                alias.capability, CHAT_CAPABILITY_ID
            )));
        }
        let trimmed = alias.alias.trim();
        if trimmed.is_empty() {
            return Err(invalid_request("route alias cannot be empty"));
        }

        self.route_aliases.get(trimmed).cloned().ok_or_else(|| {
            invalid_request(format!(
                "unknown route alias '{}'; expected one configured chat alias",
                trimmed
            ))
        })
    }

    fn resolve_chat_key(&self, key: &ChatRouteKey) -> Result<ModelTarget, GatewayError> {
        if key.capability != CHAT_CAPABILITY_ID {
            return Err(invalid_request(format!(
                "unsupported capability '{}' in route key; expected '{}'",
                key.capability, CHAT_CAPABILITY_ID
            )));
        }
        let trimmed = key.binding_id.trim();
        if trimmed.is_empty() {
            return Err(invalid_request("route key binding_id cannot be empty"));
        }

        // Current config only defines alias-based model targets.
        // Treat binding_id as the configured chat binding selector until chat bindings are explicit.
        self.route_aliases.get(trimmed).cloned().ok_or_else(|| {
            invalid_request(format!(
                "unknown route key binding_id '{}'; expected one configured chat binding selector",
                trimmed
            ))
        })
    }
}
