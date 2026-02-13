# L3-03 - Runtime And Endpoint Pseudocode
- Task Name: `body-endpoints-mvp`
- Stage: `L3`
- Date: `2026-02-12`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Runtime `start` pseudocode
```rust
fn cmd_start(cfg: RuntimeConfig) -> Result<()> {
    ensure_not_already_running()?;

    let core_child = spawn_process(cfg.core.command, cfg.core.args, cfg.core.env)?;
    wait_until_socket_ready(cfg.core.socket_path, cfg.start_timeout_ms)?;

    let std_body_child = spawn_process(cfg.std_body.command, cfg.std_body.args, cfg.std_body.env)?;
    persist_state(State {
        core_pid: core_child.id(),
        std_body_pid: std_body_child.id(),
        socket_path: cfg.core.socket_path.clone(),
        started_at: now(),
    })?;

    Ok(())
}
```

## 2) Runtime `stop` pseudocode
```rust
fn cmd_stop(cfg: RuntimeConfig) -> Result<()> {
    let state = load_state()?;
    send_exit_message(state.socket_path)?;

    wait_for_exit(state.core_pid, cfg.stop_grace_ms)?;
    terminate_if_alive(state.std_body_pid)?;
    clear_state()?;
    Ok(())
}
```

## 3) Std-body host loop pseudocode
```rust
async fn run_host(cfg: HostConfig) -> Result<()> {
    loop {
        let stream = connect_socket(&cfg.socket_path).await?;
        send_register(stream, shell_registration()) .await?;
        send_register(stream, web_registration()) .await?;

        while let Some(msg) = recv_message(stream).await? {
            match msg {
                Message::EndpointInvoke { request_id, action } => {
                    let outcome = dispatch(action).await;
                    send_endpoint_result(stream, request_id, outcome).await?;
                }
                Message::Exit => return Ok(()),
                _ => {}
            }
        }

        sleep(cfg.reconnect_backoff_ms).await;
    }
}
```

## 4) Shell handler pseudocode
```rust
async fn handle_shell(payload: Value, caps: ShellCaps) -> EndpointOutcome {
    let req = parse_shell(payload).or_rejected("invalid_payload")?;
    validate_argv(&req).or_rejected("invalid_payload")?;

    let timeout = min(req.timeout_ms.unwrap_or(caps.default_timeout_ms), caps.max_timeout_ms);
    let out_cap = min(req.stdout_max_bytes.unwrap_or(caps.default_stdout_max_bytes), caps.max_stdout_max_bytes);
    let err_cap = min(req.stderr_max_bytes.unwrap_or(caps.default_stderr_max_bytes), caps.max_stderr_max_bytes);

    let result = run_command_argv(req.argv, req.cwd, req.env, timeout, out_cap, err_cap).await;
    map_shell_result(result)
}
```

## 5) Web handler pseudocode
```rust
async fn handle_web(payload: Value, caps: WebCaps) -> EndpointOutcome {
    let req = parse_web(payload).or_rejected("invalid_payload")?;
    ensure_http_scheme(&req.url).or_rejected("unsupported_scheme")?;

    let timeout = min(req.timeout_ms.unwrap_or(caps.default_timeout_ms), caps.max_timeout_ms);
    let body_cap = min(req.response_max_bytes.unwrap_or(caps.default_response_max_bytes), caps.max_response_max_bytes);

    let result = fetch_http(req, timeout, body_cap).await;
    map_web_result(result) // 4xx/5xx => applied with status payload
}
```

## 6) Apple invoke payload shape guard pseudocode
```rust
fn validate_chat_payload(payload: &Value) -> Result<()> {
    require(payload["conversation_id"].is_string(), "conversation_id");
    require(payload["response"]["object"] == "response", "response.object");
    require(payload["response"]["output"].is_array(), "response.output");
    Ok(())
}
```

Status: `READY_FOR_REVIEW`
