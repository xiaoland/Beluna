# L3-03 - Core Pseudocode

- Task Name: `minimal-ai-gateway`
- Stage: `L3` detail: core logic pseudocode
- Date: 2026-02-08
- Status: `DRAFT_FOR_APPROVAL`

## 1) `AIGateway::infer_stream`

```text
fn infer_stream(beluna_request):
  canonical_req = request_normalizer.normalize(beluna_request)

  selected = router.select(canonical_req)
  credential = credential_provider.resolve(selected.profile.credential, selected.profile)
  adapter = adapter_registry.get(selected.profile.dialect)

  effective_caps = merge(adapter.static_capabilities, selected.profile.capability_overrides)
  capability_guard.assert_supported(canonical_req, effective_caps)

  budget_ctx = budget_enforcer.pre_dispatch(canonical_req, selected.backend_id)

  stream_factory = || {
    adapter_ctx = {
      backend_id: selected.backend_id,
      model: selected.resolved_model,
      credential,
      timeout: budget_ctx.effective_timeout,
      request_id: canonical_req.request_id,
    }
    adapter.invoke_stream(adapter_ctx, canonical_req)
  }

  invocation_stream = reliability.execute_stream(
    request = canonical_req,
    backend = selected.backend_id,
    stream_factory = stream_factory,
  )

  canonical_stream = response_normalizer.wrap(
    request_id = canonical_req.request_id,
    backend_id = selected.backend_id,
    model = selected.resolved_model,
    raw_stream = invocation_stream.stream,
  )

  final_stream = wrap_with_lifecycle(
    canonical_stream,
    on_event = |ev| {
      budget_enforcer.observe_event(ev)
      telemetry.emit(ev_to_telemetry(ev))
    },
    on_drop = || {
      if invocation_stream.cancel exists: invocation_stream.cancel()
      budget_enforcer.release(budget_ctx)
      telemetry.emit(cancelled_event)
    },
    on_terminal = || {
      budget_enforcer.release(budget_ctx)
    }
  )

  return final_stream
```

## 2) `AIGateway::infer_once`

```text
fn infer_once(req):
  stream = infer_stream(req)

  aggregate = {
    output_text = ""
    tool_calls = []
    usage = None
    finish_reason = None
  }

  for event in stream:
    match event:
      OutputTextDelta(delta) => aggregate.output_text += delta
      ToolCallReady(call) => aggregate.tool_calls.push(call)
      Usage(u) => aggregate.usage = Some(u)
      Completed(reason) => aggregate.finish_reason = Some(reason)
      Failed(err) => return Err(err)
      _ => continue

  return CanonicalFinalResponse(aggregate)
```

## 3) `RequestNormalizer::normalize`

```text
fn normalize(req):
  if req.messages.is_empty(): invalid_request("messages must not be empty")

  request_id = req.request_id.unwrap_or(generate_uuid_v7())

  for each message in req.messages:
    if message.role == tool:
      require(message.tool_call_id.is_some())
      require(message.tool_name.is_some())
      reject_if_any_part_is_image(message.parts)
    else:
      require(message.tool_call_id.is_none())
      require(message.tool_name.is_none())

  validate_tool_schema_keywords(req.tools)

  return CanonicalRequest { ...strict-mapped fields... }
```

## 4) `BackendRouter::select`

```text
fn select(req):
  backend_id = req.backend_hint.unwrap_or(config.default_backend)
  profile = config.backends.get(backend_id) or invalid_request("unknown backend")
  model = req.model_override.unwrap_or(profile.default_model)

  return SelectedBackend { backend_id, profile, resolved_model = model }
```

Rule:
- deterministic selection only; no failover/fallback in MVP.

## 5) `ReliabilityLayer::execute_stream`

```text
fn execute_stream(request, backend, stream_factory):
  attempt = 0

  loop:
    breaker.assert_allows(backend)?

    invoke_result = stream_factory()
    if invoke_result is immediate_error:
      mapped = map_error(invoke_result)
      if can_retry(mapped, attempt, emitted_output=false, emitted_tool=false):
        breaker.record_transient_failure(backend)
        sleep(backoff(attempt))
        attempt += 1
        continue
      breaker.record_terminal(backend, mapped)
      return Err(mapped)

    emitted_output = false
    emitted_tool = false

    for raw_event in invoke_result.stream:
      if consumer_dropped:
        if invoke_result.cancel exists: invoke_result.cancel()
        return Cancelled

      mapped_event = response_normalizer.map(raw_event)
      if mapped_event is OutputTextDelta or ToolCallDelta or ToolCallReady:
        emitted_output = true
      if mapped_event is ToolCallDelta or ToolCallReady:
        emitted_tool = true

      yield mapped_event

    if terminal_success:
      breaker.record_success(backend)
      return

    mapped_err = terminal_error
    if can_retry(mapped_err, attempt, emitted_output, emitted_tool):
      breaker.record_transient_failure(backend)
      sleep(backoff(attempt))
      attempt += 1
      continue

    breaker.record_terminal(backend, mapped_err)
    return Err(mapped_err)
```

Retry predicate:

```text
can_retry(err, attempt, emitted_output, emitted_tool) =
  err.retryable
  && attempt < max_retries
  && (!emitted_output || retry_policy == adapter_resumable)
  && (!emitted_tool || adapter_declares_tool_retry_safe)
```

## 6) `ResponseNormalizer` Event Guard

```text
fn wrap(...):
  emitted_started = false
  emitted_terminal = false
  emitted_usage = false

  emit Started first

  for each mapped backend event:
    assert !emitted_terminal

    if event == Usage:
      if emitted_usage: ignore_or_protocol_error
      emitted_usage = true

    if event == Completed or event == Failed:
      emitted_terminal = true

    emit event

  if !emitted_terminal:
    emit Failed(ProtocolViolation: missing terminal event)
```

Status: `READY_FOR_L3_REVIEW`
