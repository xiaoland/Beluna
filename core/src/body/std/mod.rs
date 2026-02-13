use std::sync::Arc;

use anyhow::{Result, anyhow};
use tokio::sync::mpsc;

use crate::{
    body::std::payloads::{ShellLimits, WebLimits},
    cortex::SenseDelta,
    spine::EndpointRegistryPort,
};

#[cfg(any(feature = "std-shell", feature = "std-web"))]
use async_trait::async_trait;

#[cfg(feature = "std-shell")]
use crate::body::std::shell::handle_shell_invoke;

#[cfg(feature = "std-web")]
use crate::body::std::web::handle_web_invoke;

#[cfg(any(feature = "std-shell", feature = "std-web"))]
use crate::spine::{
    EndpointCapabilityDescriptor, EndpointExecutionOutcome, EndpointInvocation,
    EndpointRegistration, RouteKey, error::SpineError, ports::EndpointPort, types::CostVector,
};

pub mod payloads;
pub mod shell;
pub mod web;

pub const SHELL_AFFORDANCE_KEY: &str = "tool.shell.exec";
pub const SHELL_CAPABILITY_HANDLE: &str = "cap.std.shell";
pub const WEB_AFFORDANCE_KEY: &str = "tool.web.fetch";
pub const WEB_CAPABILITY_HANDLE: &str = "cap.std.web.fetch";

pub const SHELL_ENDPOINT_ID: &str = "ep:body:std:shell";
pub const WEB_ENDPOINT_ID: &str = "ep:body:std:web";

pub fn register_std_body_endpoints(
    registry: Arc<dyn EndpointRegistryPort>,
    sense_tx: mpsc::UnboundedSender<SenseDelta>,
    shell_enabled: bool,
    shell_limits: ShellLimits,
    web_enabled: bool,
    web_limits: WebLimits,
) -> Result<()> {
    register_shell_endpoint(
        registry.clone(),
        sense_tx.clone(),
        shell_enabled,
        shell_limits,
    )?;
    register_web_endpoint(registry, sense_tx, web_enabled, web_limits)?;
    Ok(())
}

fn register_shell_endpoint(
    registry: Arc<dyn EndpointRegistryPort>,
    sense_tx: mpsc::UnboundedSender<SenseDelta>,
    enabled: bool,
    limits: ShellLimits,
) -> Result<()> {
    if !enabled {
        return Ok(());
    }

    #[cfg(feature = "std-shell")]
    {
        let endpoint: Arc<dyn EndpointPort> = Arc::new(StdShellEndpoint { limits, sense_tx });
        registry
            .register(
                EndpointRegistration {
                    endpoint_id: SHELL_ENDPOINT_ID.to_string(),
                    descriptor: shell_registration_descriptor(),
                },
                endpoint,
            )
            .map_err(|err| anyhow!(err.to_string()))?;
        return Ok(());
    }

    #[cfg(not(feature = "std-shell"))]
    {
        let _ = (registry, sense_tx, limits);
        Err(anyhow!(
            "body.std_shell.enabled=true but core is built without feature std-shell"
        ))
    }
}

fn register_web_endpoint(
    registry: Arc<dyn EndpointRegistryPort>,
    sense_tx: mpsc::UnboundedSender<SenseDelta>,
    enabled: bool,
    limits: WebLimits,
) -> Result<()> {
    if !enabled {
        return Ok(());
    }

    #[cfg(feature = "std-web")]
    {
        let endpoint: Arc<dyn EndpointPort> = Arc::new(StdWebEndpoint { limits, sense_tx });
        registry
            .register(
                EndpointRegistration {
                    endpoint_id: WEB_ENDPOINT_ID.to_string(),
                    descriptor: web_registration_descriptor(),
                },
                endpoint,
            )
            .map_err(|err| anyhow!(err.to_string()))?;
        return Ok(());
    }

    #[cfg(not(feature = "std-web"))]
    {
        let _ = (registry, sense_tx, limits);
        Err(anyhow!(
            "body.std_web.enabled=true but core is built without feature std-web"
        ))
    }
}

#[cfg(feature = "std-shell")]
struct StdShellEndpoint {
    limits: ShellLimits,
    sense_tx: mpsc::UnboundedSender<SenseDelta>,
}

#[cfg(feature = "std-web")]
struct StdWebEndpoint {
    limits: WebLimits,
    sense_tx: mpsc::UnboundedSender<SenseDelta>,
}

#[cfg(feature = "std-shell")]
#[async_trait]
impl EndpointPort for StdShellEndpoint {
    async fn invoke(
        &self,
        invocation: EndpointInvocation,
    ) -> Result<EndpointExecutionOutcome, SpineError> {
        let action = invocation.action;
        let request_id = format!("builtin-shell:{}", action.action_id);
        let output = handle_shell_invoke(&request_id, &action, &self.limits).await;
        if let Some(sense) = output.sense {
            let _ = self.sense_tx.send(sense);
        }

        Ok(output.outcome)
    }
}

#[cfg(feature = "std-web")]
#[async_trait]
impl EndpointPort for StdWebEndpoint {
    async fn invoke(
        &self,
        invocation: EndpointInvocation,
    ) -> Result<EndpointExecutionOutcome, SpineError> {
        let action = invocation.action;
        let request_id = format!("builtin-web:{}", action.action_id);
        let output = handle_web_invoke(&request_id, &action, &self.limits).await;
        if let Some(sense) = output.sense {
            let _ = self.sense_tx.send(sense);
        }

        Ok(output.outcome)
    }
}

#[cfg(feature = "std-shell")]
fn shell_registration_descriptor() -> EndpointCapabilityDescriptor {
    EndpointCapabilityDescriptor {
        route: RouteKey {
            affordance_key: SHELL_AFFORDANCE_KEY.to_string(),
            capability_handle: SHELL_CAPABILITY_HANDLE.to_string(),
        },
        payload_schema: serde_json::json!({
            "type": "object",
            "required": ["argv"],
            "properties": {
                "argv": {"type": "array", "items": {"type": "string"}},
                "cwd": {"type": ["string", "null"]},
                "env": {"type": "object", "additionalProperties": {"type": "string"}},
                "timeout_ms": {"type": "integer", "minimum": 1},
                "stdout_max_bytes": {"type": "integer", "minimum": 1},
                "stderr_max_bytes": {"type": "integer", "minimum": 1}
            }
        }),
        max_payload_bytes: 65_536,
        default_cost: CostVector {
            survival_micro: 500,
            time_ms: 2_000,
            io_units: 4,
            token_units: 0,
        },
        metadata: Default::default(),
    }
}

#[cfg(feature = "std-web")]
fn web_registration_descriptor() -> EndpointCapabilityDescriptor {
    EndpointCapabilityDescriptor {
        route: RouteKey {
            affordance_key: WEB_AFFORDANCE_KEY.to_string(),
            capability_handle: WEB_CAPABILITY_HANDLE.to_string(),
        },
        payload_schema: serde_json::json!({
            "type": "object",
            "required": ["url"],
            "properties": {
                "url": {"type": "string"},
                "method": {"type": ["string", "null"]},
                "headers": {"type": "object", "additionalProperties": {"type": "string"}, "default": {}},
                "body_text": {"type": ["string", "null"]},
                "timeout_ms": {"type": "integer", "minimum": 1},
                "response_max_bytes": {"type": "integer", "minimum": 1}
            }
        }),
        max_payload_bytes: 65_536,
        default_cost: CostVector {
            survival_micro: 450,
            time_ms: 2_500,
            io_units: 3,
            token_units: 0,
        },
        metadata: Default::default(),
    }
}
