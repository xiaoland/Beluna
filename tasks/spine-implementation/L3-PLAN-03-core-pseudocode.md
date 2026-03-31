# L3-03 - Core Pseudocode
- Task Name: `spine-implementation`
- Stage: `L3` detail: core algorithms
- Date: `2026-02-12`
- Status: `IMPLEMENTED`

## 1) Registry Registration

```text
fn register(registration, endpoint):
  assert endpoint_id non-empty
  assert route.affordance_key non-empty
  assert route.capability_handle non-empty

  lock write state

  if route exists:
    return RouteConflict

  siblings = all entries with same affordance_key
  for each sibling:
    if payload_schema/max_payload_bytes/default_cost mismatch:
      return RegistrationInvalid

  insert registration + endpoint
  version += 1
  return Ok
```

## 2) Catalog Snapshot

```text
fn catalog_snapshot():
  lock read state
  clone all descriptors
  sort by (affordance_key, capability_handle)
  return SpineCapabilityCatalog { version, entries }
```

## 3) Serialized Execute

```text
async fn execute_serialized(batch):
  prevalidate action_id/reserve_entry_id

  events = []
  for (index, action) in batch.actions:
    event = await invoke_one(action)
    events.push(seq_no = index + 1, event)

  return report(mode, events, replay_cursor)
```

## 4) Best-Effort Execute

```text
async fn execute_best_effort(batch):
  prevalidate action_id/reserve_entry_id

  futures = for each (index, action):
    async { (index + 1, await invoke_one(action)) }

  events = await join_all(futures)
  sort events by seq_no
  return report(mode, events, replay_cursor)
```

## 5) Invoke-One Mapping

```text
async fn invoke_one(action):
  route = (action.affordance_key, action.capability_handle)
  endpoint = registry.resolve(route)

  if endpoint missing:
    return ActionRejected(reason_code = "route_not_found")

  result = await endpoint.invoke(action)

  match result:
    Applied(actual_cost_micro, reference_id) -> ActionApplied
    Rejected(reason_code, reference_id) -> ActionRejected
    Deferred(reason_code) -> ActionDeferred
    Err(_) -> ActionRejected(reason_code = "endpoint_error")
```

## 6) Catalog Bridge To Cortex

```text
fn to_cortex_catalog(spine_catalog):
  group entries by affordance_key

  for each affordance group:
    descriptor = first entry
    handles = sorted-dedup capability handles

    emit AffordanceCapability {
      affordance_key,
      allowed_capability_handles: handles,
      payload_schema: descriptor.payload_schema,
      max_payload_bytes: descriptor.max_payload_bytes,
      default_resources: descriptor.default_cost mapped
    }

  return CapabilityCatalog(version = "spine:v{catalog.version}")
```

## 7) Runtime Loop Skeleton (UnixSocket Only)

```text
async fn server_run(config):
  build cortex pipeline/reactor
  build continuity engine (routing spine executor in defaults)

  create adapter ingress channel
  spawn unix_socket_adapter.run(message_tx, shutdown_token)

  loop select:
    signal -> graceful stop
    adapter message -> update assembler; on Sense enqueue ReactionInput
    reactor result -> await continuity.process_attempts(...)

  cancel shutdown token
  await adapter task
  await reactor task
```

Status: `DONE`
