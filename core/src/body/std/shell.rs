use std::process::Stdio;

use tokio::{
    process::Command,
    time::{Duration, timeout},
};

use crate::{
    body::std::payloads::{ShellExecRequest, ShellLimits},
    cortex::SenseDelta,
    spine::types::{AdmittedAction, EndpointExecutionOutcome},
};

pub struct ShellHandlerOutput {
    pub outcome: EndpointExecutionOutcome,
    pub sense: Option<SenseDelta>,
}

pub async fn handle_shell_invoke(
    request_id: &str,
    action: &AdmittedAction,
    limits: &ShellLimits,
) -> ShellHandlerOutput {
    let request: ShellExecRequest = match serde_json::from_value(action.normalized_payload.clone())
    {
        Ok(request) => request,
        Err(_) => {
            return ShellHandlerOutput {
                outcome: EndpointExecutionOutcome::Rejected {
                    reason_code: "invalid_payload".to_string(),
                    reference_id: format!(
                        "body.std.shell:invalid_payload:{}",
                        action.neural_signal_id
                    ),
                },
                sense: None,
            };
        }
    };

    if request.argv.is_empty() || request.argv[0].trim().is_empty() {
        return ShellHandlerOutput {
            outcome: EndpointExecutionOutcome::Rejected {
                reason_code: "invalid_payload".to_string(),
                reference_id: format!("body.std.shell:missing_argv:{}", action.neural_signal_id),
            },
            sense: None,
        };
    }

    let timeout_ms = request.timeout_ms(limits);
    let stdout_cap = request.stdout_max_bytes(limits);
    let stderr_cap = request.stderr_max_bytes(limits);

    let mut command = Command::new(&request.argv[0]);
    if request.argv.len() > 1 {
        command.args(&request.argv[1..]);
    }
    if let Some(cwd) = &request.cwd {
        command.current_dir(cwd);
    }
    if !request.env.is_empty() {
        command.envs(&request.env);
    }

    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    command.kill_on_drop(true);

    let output = match command.spawn() {
        Ok(child) => match timeout(Duration::from_millis(timeout_ms), child.wait_with_output())
            .await
        {
            Ok(Ok(output)) => output,
            Ok(Err(_)) => {
                return ShellHandlerOutput {
                    outcome: EndpointExecutionOutcome::Rejected {
                        reason_code: "exec_failure".to_string(),
                        reference_id: format!(
                            "body.std.shell:wait_failure:{}",
                            action.neural_signal_id
                        ),
                    },
                    sense: None,
                };
            }
            Err(_) => {
                return ShellHandlerOutput {
                    outcome: EndpointExecutionOutcome::Rejected {
                        reason_code: "timeout".to_string(),
                        reference_id: format!("body.std.shell:timeout:{}", action.neural_signal_id),
                    },
                    sense: None,
                };
            }
        },
        Err(_) => {
            return ShellHandlerOutput {
                outcome: EndpointExecutionOutcome::Rejected {
                    reason_code: "exec_failure".to_string(),
                    reference_id: format!(
                        "body.std.shell:spawn_failure:{}",
                        action.neural_signal_id
                    ),
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
                reference_id: format!(
                    "body.std.shell:non_zero_exit:{}:{}",
                    exit_code, action.neural_signal_id
                ),
            },
            sense: Some(SenseDelta {
                sense_id: format!("sense:shell:{request_id}"),
                source: "body.std.shell".to_string(),
                payload: serde_json::json!({
                    "kind": "shell_result",
                    "neural_signal_id": action.neural_signal_id,
                    "capability_instance_id": action.capability_instance_id,
                    "endpoint_id": action.endpoint_id,
                    "capability_id": action.capability_id,
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
            actual_cost_micro: action.reserved_cost.survival_micro.max(0),
            reference_id: format!("body.std.shell:applied:{}", action.neural_signal_id),
        },
        sense: Some(SenseDelta {
            sense_id: format!("sense:shell:{request_id}"),
            source: "body.std.shell".to_string(),
            payload: serde_json::json!({
                "kind": "shell_result",
                "neural_signal_id": action.neural_signal_id,
                "capability_instance_id": action.capability_instance_id,
                "endpoint_id": action.endpoint_id,
                "capability_id": action.capability_id,
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
    use crate::spine::types::{AdmittedAction, CostVector, EndpointExecutionOutcome};

    use super::{ShellLimits, handle_shell_invoke};

    fn build_action(neural_signal_id: &str, payload: serde_json::Value) -> AdmittedAction {
        AdmittedAction {
            neural_signal_id: neural_signal_id.to_string(),
            capability_instance_id: "shell.instance".to_string(),
            source_attempt_id: "att:1".to_string(),
            reserve_entry_id: "res:1".to_string(),
            cost_attribution_id: "cost:1".to_string(),
            endpoint_id: "ep:body:std:shell".to_string(),
            capability_id: "tool.shell.exec".to_string(),
            normalized_payload: payload,
            reserved_cost: CostVector {
                survival_micro: 123,
                time_ms: 1,
                io_units: 1,
                token_units: 0,
            },
            degraded: false,
            degradation_profile_id: None,
            admission_cycle: 1,
            metadata: Default::default(),
        }
    }

    #[tokio::test]
    async fn rejects_invalid_payload() {
        let action = build_action(
            "action:invalid",
            serde_json::json!({"url": "https://example.com"}),
        );
        let output = handle_shell_invoke("req:1", &action, &ShellLimits::default()).await;

        assert!(matches!(
            output.outcome,
            EndpointExecutionOutcome::Rejected { ref reason_code, .. } if reason_code == "invalid_payload"
        ));
        assert!(output.sense.is_none());
    }

    #[tokio::test]
    async fn applies_on_zero_exit() {
        let action = build_action(
            "action:ok",
            serde_json::json!({
                "argv": ["/bin/sh", "-c", "printf 'hello'"],
                "stdout_max_bytes": 32,
                "stderr_max_bytes": 32
            }),
        );

        let output = handle_shell_invoke("req:ok", &action, &ShellLimits::default()).await;

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
        let action = build_action(
            "action:bad",
            serde_json::json!({
                "argv": ["/bin/sh", "-c", "echo oops 1>&2; exit 7"],
                "stdout_max_bytes": 32,
                "stderr_max_bytes": 32
            }),
        );

        let output = handle_shell_invoke("req:bad", &action, &ShellLimits::default()).await;

        assert!(matches!(
            output.outcome,
            EndpointExecutionOutcome::Rejected { ref reason_code, .. } if reason_code == "non_zero_exit"
        ));

        let sense = output.sense.expect("sense should be emitted");
        assert_eq!(sense.payload["success"], serde_json::json!(false));
        assert_eq!(sense.payload["exit_code"], serde_json::json!(7));
    }
}
