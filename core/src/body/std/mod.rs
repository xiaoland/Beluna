use std::sync::Arc;

use anyhow::{Result, anyhow};

use crate::{
    body::std::payloads::{ShellLimits, WebLimits},
    ingress::SenseIngress,
    spine::runtime::Spine,
    types::{Act, Sense},
};

#[cfg(any(feature = "std-shell", feature = "std-web"))]
use async_trait::async_trait;

#[cfg(feature = "std-shell")]
use crate::body::std::shell::handle_shell_invoke;

#[cfg(feature = "std-web")]
use crate::body::std::web::handle_web_invoke;

#[cfg(any(feature = "std-shell", feature = "std-web"))]
use crate::spine::{
    Endpoint, EndpointBinding, EndpointCapabilityDescriptor, EndpointExecutionOutcome, RouteKey,
    error::SpineError, types::CostVector,
};

pub mod payloads;
pub mod shell;
pub mod web;

pub const SHELL_ENDPOINT_NAME: &str = "std-shell";
pub const WEB_ENDPOINT_NAME: &str = "std-web";
pub const SHELL_CAPABILITY_ID: &str = "tool.shell.exec";
pub const WEB_CAPABILITY_ID: &str = "tool.web.fetch";

pub fn register_std_body_endpoints(
    spine: Arc<Spine>,
    sense_ingress: SenseIngress,
    shell_enabled: bool,
    shell_limits: ShellLimits,
    web_enabled: bool,
    web_limits: WebLimits,
) -> Result<()> {
    register_shell_endpoint(
        Arc::clone(&spine),
        sense_ingress.clone(),
        shell_enabled,
        shell_limits,
    )?;
    register_web_endpoint(spine, sense_ingress, web_enabled, web_limits)?;
    Ok(())
}

fn register_shell_endpoint(
    spine: Arc<Spine>,
    sense_ingress: SenseIngress,
    enabled: bool,
    limits: ShellLimits,
) -> Result<()> {
    if !enabled {
        return Ok(());
    }

    #[cfg(feature = "std-shell")]
    {
        let endpoint: Arc<dyn Endpoint> = Arc::new(StdShellEndpoint {
            limits,
            sense_ingress,
        });
        spine
            .add_endpoint(
                SHELL_ENDPOINT_NAME,
                EndpointBinding::Inline(endpoint),
                vec![shell_registration_descriptor()],
            )
            .map_err(|err| anyhow!(err.to_string()))?;
        return Ok(());
    }

    #[cfg(not(feature = "std-shell"))]
    {
        let _ = (spine, sense_ingress, limits);
        Err(anyhow!(
            "body.std_shell.enabled=true but core is built without feature std-shell"
        ))
    }
}

fn register_web_endpoint(
    spine: Arc<Spine>,
    sense_ingress: SenseIngress,
    enabled: bool,
    limits: WebLimits,
) -> Result<()> {
    if !enabled {
        return Ok(());
    }

    #[cfg(feature = "std-web")]
    {
        let endpoint: Arc<dyn Endpoint> = Arc::new(StdWebEndpoint {
            limits,
            sense_ingress,
        });
        spine
            .add_endpoint(
                WEB_ENDPOINT_NAME,
                EndpointBinding::Inline(endpoint),
                vec![web_registration_descriptor()],
            )
            .map_err(|err| anyhow!(err.to_string()))?;
        return Ok(());
    }

    #[cfg(not(feature = "std-web"))]
    {
        let _ = (spine, sense_ingress, limits);
        Err(anyhow!(
            "body.std_web.enabled=true but core is built without feature std-web"
        ))
    }
}

#[cfg(feature = "std-shell")]
struct StdShellEndpoint {
    limits: ShellLimits,
    sense_ingress: SenseIngress,
}

#[cfg(feature = "std-web")]
struct StdWebEndpoint {
    limits: WebLimits,
    sense_ingress: SenseIngress,
}

#[cfg(feature = "std-shell")]
#[async_trait]
impl Endpoint for StdShellEndpoint {
    async fn invoke(&self, act: Act) -> Result<EndpointExecutionOutcome, SpineError> {
        let request_id = format!("builtin-shell:{}", act.act_id);
        let output = handle_shell_invoke(&request_id, &act, &self.limits).await;
        if let Some(sense) = output.sense {
            let _ = self.sense_ingress.send(Sense::Domain(sense)).await;
        }

        Ok(output.outcome)
    }
}

#[cfg(feature = "std-web")]
#[async_trait]
impl Endpoint for StdWebEndpoint {
    async fn invoke(&self, act: Act) -> Result<EndpointExecutionOutcome, SpineError> {
        let request_id = format!("builtin-web:{}", act.act_id);
        let output = handle_web_invoke(&request_id, &act, &self.limits).await;
        if let Some(sense) = output.sense {
            let _ = self.sense_ingress.send(Sense::Domain(sense)).await;
        }

        Ok(output.outcome)
    }
}

#[cfg(feature = "std-shell")]
fn shell_registration_descriptor() -> EndpointCapabilityDescriptor {
    EndpointCapabilityDescriptor {
        route: RouteKey {
            endpoint_id: SHELL_ENDPOINT_NAME.to_string(),
            capability_id: SHELL_CAPABILITY_ID.to_string(),
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
            endpoint_id: WEB_ENDPOINT_NAME.to_string(),
            capability_id: WEB_CAPABILITY_ID.to_string(),
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
