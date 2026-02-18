use std::sync::Arc;

use anyhow::{Result, anyhow};

use crate::{
    body::payloads::{ShellLimits, WebLimits},
    spine::{
        EndpointCapabilityDescriptor, RouteKey,
        adapters::inline::{InlineEndpointRuntimeHandles, SpineInlineAdapter},
        types::CostVector,
    },
};

#[cfg(feature = "std-shell")]
use crate::body::shell::handle_shell_invoke;

#[cfg(feature = "std-web")]
use crate::body::web::handle_web_invoke;

pub mod payloads;
pub mod shell;
pub mod web;

pub const SHELL_ENDPOINT_NAME: &str = "std-shell";
pub const WEB_ENDPOINT_NAME: &str = "std-web";
pub const SHELL_CAPABILITY_ID: &str = "tool.shell.exec";
pub const WEB_CAPABILITY_ID: &str = "tool.web.fetch";

/// Inline Body Endpoints are built into Core, started by `main`, and attached through Spine inline adapter.
pub fn register_inline_body_endpoints(
    inline_adapter: Arc<SpineInlineAdapter>,
    shell_enabled: bool,
    shell_limits: ShellLimits,
    web_enabled: bool,
    web_limits: WebLimits,
) -> Result<()> {
    if shell_enabled {
        start_std_shell_inline_endpoint(Arc::clone(&inline_adapter), shell_limits)?;
    }
    if web_enabled {
        start_std_web_inline_endpoint(inline_adapter, web_limits)?;
    }
    Ok(())
}

fn start_std_shell_inline_endpoint(
    inline_adapter: Arc<SpineInlineAdapter>,
    limits: ShellLimits,
) -> Result<()> {
    #[cfg(feature = "std-shell")]
    {
        let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<()>>();
        std::thread::Builder::new()
            .name("beluna-inline-std-shell".to_string())
            .spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build();
                let Ok(runtime) = runtime else {
                    let _ = ready_tx.send(Err(anyhow!(
                        "failed to build tokio runtime for inline std-shell worker"
                    )));
                    return;
                };

                runtime.block_on(async move {
                    let handles = inline_adapter
                        .attach_inline_endpoint(
                            SHELL_ENDPOINT_NAME.to_string(),
                            vec![shell_registration_descriptor()],
                        )
                        .await;

                    match handles {
                        Ok(handles) => {
                            let _ = ready_tx.send(Ok(()));
                            run_shell_worker(handles, limits).await;
                        }
                        Err(err) => {
                            let _ = ready_tx.send(Err(err));
                        }
                    }
                });
            })
            .map_err(|err| anyhow!("failed to spawn inline std-shell worker thread: {err}"))?;

        ready_rx
            .recv()
            .map_err(|_| anyhow!("inline std-shell worker startup channel closed"))??;
        return Ok(());
    }

    #[cfg(not(feature = "std-shell"))]
    {
        let _ = (inline_adapter, limits);
        Err(anyhow!(
            "body.std_shell.enabled=true but core is built without feature std-shell"
        ))
    }
}

fn start_std_web_inline_endpoint(
    inline_adapter: Arc<SpineInlineAdapter>,
    limits: WebLimits,
) -> Result<()> {
    #[cfg(feature = "std-web")]
    {
        let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<()>>();
        std::thread::Builder::new()
            .name("beluna-inline-std-web".to_string())
            .spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build();
                let Ok(runtime) = runtime else {
                    let _ = ready_tx.send(Err(anyhow!(
                        "failed to build tokio runtime for inline std-web worker"
                    )));
                    return;
                };

                runtime.block_on(async move {
                    let handles = inline_adapter
                        .attach_inline_endpoint(
                            WEB_ENDPOINT_NAME.to_string(),
                            vec![web_registration_descriptor()],
                        )
                        .await;

                    match handles {
                        Ok(handles) => {
                            let _ = ready_tx.send(Ok(()));
                            run_web_worker(handles, limits).await;
                        }
                        Err(err) => {
                            let _ = ready_tx.send(Err(err));
                        }
                    }
                });
            })
            .map_err(|err| anyhow!("failed to spawn inline std-web worker thread: {err}"))?;

        ready_rx
            .recv()
            .map_err(|_| anyhow!("inline std-web worker startup channel closed"))??;
        return Ok(());
    }

    #[cfg(not(feature = "std-web"))]
    {
        let _ = (inline_adapter, limits);
        Err(anyhow!(
            "body.std_web.enabled=true but core is built without feature std-web"
        ))
    }
}

#[cfg(feature = "std-shell")]
async fn run_shell_worker(mut handles: InlineEndpointRuntimeHandles, limits: ShellLimits) {
    while let Some(act) = handles.act_rx.recv().await {
        let request_id = format!("builtin-shell:{}", act.act_id);
        let output = handle_shell_invoke(&request_id, act.as_ref(), &limits).await;
        if let Some(sense) = output.sense
            && handles.sense_tx.send(Arc::new(sense)).await.is_err()
        {
            break;
        }
    }
}

#[cfg(feature = "std-web")]
async fn run_web_worker(mut handles: InlineEndpointRuntimeHandles, limits: WebLimits) {
    while let Some(act) = handles.act_rx.recv().await {
        let request_id = format!("builtin-web:{}", act.act_id);
        let output = handle_web_invoke(&request_id, act.as_ref(), &limits).await;
        if let Some(sense) = output.sense
            && handles.sense_tx.send(Arc::new(sense)).await.is_err()
        {
            break;
        }
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
