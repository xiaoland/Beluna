use std::process::Stdio;

use tokio::{
    process::Command,
    time::{Duration, timeout},
};

use crate::{
    body::{
        SHELL_SENSE_EXEC_RESULT_ID,
        payloads::{ShellExecRequest, ShellLimits},
    },
    spine::adapters::inline::InlineSenseDatum,
    spine::types::EndpointExecutionOutcome,
    types::Act,
};

pub struct ShellHandlerOutput {
    pub outcome: EndpointExecutionOutcome,
    pub sense: Option<InlineSenseDatum>,
}

pub async fn handle_shell_invoke(
    _request_id: &str,
    act: &Act,
    limits: &ShellLimits,
) -> ShellHandlerOutput {
    let parse_result: Result<ShellExecRequest, _> = serde_json::from_value(act.payload.clone());
    let command_request = match parse_result {
        Ok(request) => request,
        Err(_) => {
            return ShellHandlerOutput {
                outcome: EndpointExecutionOutcome::Rejected {
                    reason_code: "invalid_payload".to_string(),
                    reference_id: format!("body.std.shell:invalid_payload:{}", act.act_instance_id),
                },
                sense: None,
            };
        }
    };

    if command_request.argv.is_empty() || command_request.argv[0].trim().is_empty() {
        return ShellHandlerOutput {
            outcome: EndpointExecutionOutcome::Rejected {
                reason_code: "invalid_payload".to_string(),
                reference_id: format!("body.std.shell:missing_argv:{}", act.act_instance_id),
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
                            reference_id: format!(
                                "body.std.shell:wait_failure:{}",
                                act.act_instance_id
                            ),
                        },
                        sense: None,
                    };
                }
                Err(_) => {
                    return ShellHandlerOutput {
                        outcome: EndpointExecutionOutcome::Rejected {
                            reason_code: "timeout".to_string(),
                            reference_id: format!("body.std.shell:timeout:{}", act.act_instance_id),
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
                    reference_id: format!("body.std.shell:spawn_failure:{}", act.act_instance_id),
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
                    exit_code, act.act_instance_id
                ),
            },
            sense: Some(InlineSenseDatum {
                sense_instance_id: uuid::Uuid::new_v4().to_string(),
                neural_signal_descriptor_id: SHELL_SENSE_EXEC_RESULT_ID.to_string(),
                payload: build_shell_result_payload(
                    act,
                    exit_code,
                    &stdout_text,
                    &stderr_text,
                    stdout_truncated,
                    stderr_truncated,
                    false,
                ),
                weight: 1.0,
                act_instance_id: Some(act.act_instance_id.clone()),
            }),
        };
    }

    ShellHandlerOutput {
        outcome: EndpointExecutionOutcome::Applied {
            actual_cost_micro: 0,
            reference_id: format!("body.std.shell:applied:{}", act.act_instance_id),
        },
        sense: Some(InlineSenseDatum {
            sense_instance_id: uuid::Uuid::new_v4().to_string(),
            neural_signal_descriptor_id: SHELL_SENSE_EXEC_RESULT_ID.to_string(),
            payload: build_shell_result_payload(
                act,
                exit_code,
                &stdout_text,
                &stderr_text,
                stdout_truncated,
                stderr_truncated,
                true,
            ),
            weight: 0.0,
            act_instance_id: Some(act.act_instance_id.clone()),
        }),
    }
}

fn build_shell_result_payload(
    act: &Act,
    exit_code: i32,
    stdout_text: &str,
    stderr_text: &str,
    stdout_truncated: bool,
    stderr_truncated: bool,
    success: bool,
) -> String {
    format!(
        concat!(
            "shell_result act_instance_id={}; neural_signal_descriptor_id={}; ",
            "success={}; exit_code={}; stdout_truncated={}; stderr_truncated={}\n",
            "stdout:\n{}\n",
            "stderr:\n{}"
        ),
        act.act_instance_id,
        act.neural_signal_descriptor_id,
        success,
        exit_code,
        stdout_truncated,
        stderr_truncated,
        stdout_text,
        stderr_text
    )
}

fn truncate_to_text(bytes: &[u8], cap: usize) -> (String, bool) {
    if bytes.len() <= cap {
        return (String::from_utf8_lossy(bytes).to_string(), false);
    }

    (String::from_utf8_lossy(&bytes[..cap]).to_string(), true)
}
