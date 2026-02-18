use std::process::Stdio;

use tokio::{
    process::Command,
    time::{Duration, timeout},
};

use crate::{
    body::payloads::{ShellExecRequest, ShellLimits},
    spine::types::EndpointExecutionOutcome,
    types::{Act, SenseDatum},
};

pub struct ShellHandlerOutput {
    pub outcome: EndpointExecutionOutcome,
    pub sense: Option<SenseDatum>,
}

pub async fn handle_shell_invoke(
    _request_id: &str,
    act: &Act,
    limits: &ShellLimits,
) -> ShellHandlerOutput {
    let parse_result: Result<ShellExecRequest, _> =
        serde_json::from_value(act.normalized_payload.clone());
    let command_request = match parse_result {
        Ok(request) => request,
        Err(_) => {
            return ShellHandlerOutput {
                outcome: EndpointExecutionOutcome::Rejected {
                    reason_code: "invalid_payload".to_string(),
                    reference_id: format!("body.std.shell:invalid_payload:{}", act.act_id),
                },
                sense: None,
            };
        }
    };

    if command_request.argv.is_empty() || command_request.argv[0].trim().is_empty() {
        return ShellHandlerOutput {
            outcome: EndpointExecutionOutcome::Rejected {
                reason_code: "invalid_payload".to_string(),
                reference_id: format!("body.std.shell:missing_argv:{}", act.act_id),
            },
            sense: None,
        };
    }

    let timeout_ms = command_request.timeout_ms(limits);
    let stdout_cap = command_request.stdout_max_bytes(limits);
    let stderr_cap = command_request.stderr_max_bytes(limits);

    let mut command = Command::new(&command_request.argv[0]);
    if command_request.argv.len() > 1 {
        command.args(&command_request.argv[1..]);
    }
    if let Some(cwd) = &command_request.cwd {
        command.current_dir(cwd);
    }
    if !command_request.env.is_empty() {
        command.envs(&command_request.env);
    }

    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    command.kill_on_drop(true);

    let output = match command.spawn() {
        Ok(child) => {
            match timeout(Duration::from_millis(timeout_ms), child.wait_with_output()).await {
                Ok(Ok(output)) => output,
                Ok(Err(_)) => {
                    return ShellHandlerOutput {
                        outcome: EndpointExecutionOutcome::Rejected {
                            reason_code: "exec_failure".to_string(),
                            reference_id: format!("body.std.shell:wait_failure:{}", act.act_id),
                        },
                        sense: None,
                    };
                }
                Err(_) => {
                    return ShellHandlerOutput {
                        outcome: EndpointExecutionOutcome::Rejected {
                            reason_code: "timeout".to_string(),
                            reference_id: format!("body.std.shell:timeout:{}", act.act_id),
                        },
                        sense: None,
                    };
                }
            }
        }
        Err(_) => {
            return ShellHandlerOutput {
                outcome: EndpointExecutionOutcome::Rejected {
                    reason_code: "exec_failure".to_string(),
                    reference_id: format!("body.std.shell:spawn_failure:{}", act.act_id),
                },
                sense: None,
            };
        }
    };

    let exit_code = output.status.code().unwrap_or(-1);
    let (stdout_text, stdout_truncated) = truncate_to_text(&output.stdout, stdout_cap);
    let (stderr_text, stderr_truncated) = truncate_to_text(&output.stderr, stderr_cap);

    if !output.status.success() {
        return ShellHandlerOutput {
            outcome: EndpointExecutionOutcome::Rejected {
                reason_code: "non_zero_exit".to_string(),
                reference_id: format!("body.std.shell:non_zero_exit:{}:{}", exit_code, act.act_id),
            },
            sense: Some(SenseDatum {
                sense_id: uuid::Uuid::new_v4().to_string(),
                source: "body.std.shell".to_string(),
                payload: serde_json::json!({
                    "kind": "shell_result",
                    "act_id": act.act_id,
                    "capability_instance_id": act.capability_instance_id,
                    "endpoint_id": act.body_endpoint_name,
                    "capability_id": act.capability_id,
                    "exit_code": exit_code,
                    "stdout_text": stdout_text,
                    "stderr_text": stderr_text,
                    "stdout_truncated": stdout_truncated,
                    "stderr_truncated": stderr_truncated,
                    "success": false
                }),
            }),
        };
    }

    ShellHandlerOutput {
        outcome: EndpointExecutionOutcome::Applied {
            actual_cost_micro: act.requested_resources.survival_micro.max(0),
            reference_id: format!("body.std.shell:applied:{}", act.act_id),
        },
        sense: Some(SenseDatum {
            sense_id: uuid::Uuid::new_v4().to_string(),
            source: "body.std.shell".to_string(),
            payload: serde_json::json!({
                "kind": "shell_result",
                "act_id": act.act_id,
                "capability_instance_id": act.capability_instance_id,
                "endpoint_id": act.body_endpoint_name,
                "capability_id": act.capability_id,
                "exit_code": exit_code,
                "stdout_text": stdout_text,
                "stderr_text": stderr_text,
                "stdout_truncated": stdout_truncated,
                "stderr_truncated": stderr_truncated,
                "success": true
            }),
        }),
    }
}

fn truncate_to_text(bytes: &[u8], cap: usize) -> (String, bool) {
    if bytes.len() <= cap {
        return (String::from_utf8_lossy(bytes).to_string(), false);
    }

    (String::from_utf8_lossy(&bytes[..cap]).to_string(), true)
}

#[cfg(test)]
mod tests {
    use crate::{
        spine::types::EndpointExecutionOutcome,
        types::{Act, RequestedResources},
    };

    use super::{ShellLimits, handle_shell_invoke};

    fn build_request(act_id: &str, payload: serde_json::Value) -> Act {
        Act {
            act_id: act_id.to_string(),
            based_on: vec!["sense:1".to_string()],
            body_endpoint_name: "ep:body:std:shell".to_string(),
            capability_id: "tool.shell.exec".to_string(),
            capability_instance_id: "shell.instance".to_string(),
            normalized_payload: payload,
            requested_resources: RequestedResources {
                survival_micro: 123,
                time_ms: 1,
                io_units: 1,
                token_units: 0,
            },
        }
    }

    #[tokio::test]
    async fn rejects_invalid_payload() {
        let request = build_request(
            "act:invalid",
            serde_json::json!({"url": "https://example.com"}),
        );
        let output = handle_shell_invoke("req:1", &request, &ShellLimits::default()).await;

        assert!(matches!(
            output.outcome,
            EndpointExecutionOutcome::Rejected { ref reason_code, .. } if reason_code == "invalid_payload"
        ));
        assert!(output.sense.is_none());
    }

    #[tokio::test]
    async fn applies_on_zero_exit() {
        let request = build_request(
            "act:ok",
            serde_json::json!({
                "argv": ["/bin/sh", "-c", "printf 'hello'"],
                "stdout_max_bytes": 32,
                "stderr_max_bytes": 32
            }),
        );

        let output = handle_shell_invoke("req:ok", &request, &ShellLimits::default()).await;

        assert!(matches!(
            output.outcome,
            EndpointExecutionOutcome::Applied {
                actual_cost_micro: 123,
                ..
            }
        ));

        let sense = output.sense.expect("sense should be emitted");
        assert_eq!(sense.source, "body.std.shell");
        assert_eq!(sense.payload["success"], serde_json::json!(true));
        assert_eq!(sense.payload["stdout_text"], serde_json::json!("hello"));
    }

    #[tokio::test]
    async fn rejects_non_zero_exit_with_sense() {
        let request = build_request(
            "act:bad",
            serde_json::json!({
                "argv": ["/bin/sh", "-c", "echo oops 1>&2; exit 7"],
                "stdout_max_bytes": 32,
                "stderr_max_bytes": 32
            }),
        );

        let output = handle_shell_invoke("req:bad", &request, &ShellLimits::default()).await;

        assert!(matches!(
            output.outcome,
            EndpointExecutionOutcome::Rejected { ref reason_code, .. } if reason_code == "non_zero_exit"
        ));

        let sense = output.sense.expect("sense should be emitted");
        assert_eq!(sense.payload["success"], serde_json::json!(false));
        assert_eq!(sense.payload["exit_code"], serde_json::json!(7));
    }
}
