# L2-03 - Routing, Catalog, And Dispatch Algorithms
- Task Name: `spine-implementation`
- Stage: `L2` detailed file
- Date: `2026-02-12`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Registration Algorithm

`InMemoryEndpointRegistry::register(registration, endpoint)`

Pseudo-flow:
```text
1. validate endpoint_id non-empty.
2. validate route.affordance_key and route.capability_handle non-empty.
3. acquire write lock.
4. if exact route already exists -> RouteConflict error.
5. collect existing entries with same affordance_key.
6. enforce affordance consistency:
   - payload_schema identical
   - max_payload_bytes identical
   - default_cost identical
   if mismatch -> RegistrationInvalid error.
7. insert (route -> {registration, endpoint}).
8. version += 1.
9. return Ok.
```

## 2) Catalog Snapshot Algorithm

`catalog_snapshot()`

Pseudo-flow:
```text
1. acquire read lock.
2. clone descriptors from all entries.
3. sort by (affordance_key, capability_handle).
4. emit SpineCapabilityCatalog { version, entries }.
```

Determinism requirement:
- same registration state must always produce byte-stable ordering.

## 3) Execution Algorithm (Mechanical Routing)

`RoutingSpineExecutor::execute_admitted(admitted_batch)`

### 3.1 Shared pre-check
```text
1. validate each action has non-empty action_id/reserve_entry_id.
2. if invalid -> InvalidBatch error (whole call).
3. branch by mode:
   - SerializedDeterministic
   - BestEffortReplayable
```

### 3.2 SerializedDeterministic mode
```text
for action in admitted.actions in input order:
  seq_no = index + 1
  route = (action.affordance_key, action.capability_handle)
  endpoint = registry.resolve(route)

  if endpoint missing:
    emit ActionRejected(reason_code="route_not_found")
    continue

  invoke endpoint asynchronously (await)
  map outcome:
    Applied  -> ActionApplied
    Rejected -> ActionRejected
    Deferred -> ActionDeferred

  if invoke returns Err:
    emit ActionRejected(reason_code="endpoint_error")
```

### 3.3 BestEffortReplayable mode
```text
1. pre-assign seq_no by input order.
2. dispatch each action concurrently (join_all / bounded fanout).
3. each task returns (seq_no, mapped_event).
4. collect and sort by seq_no.
5. output ordered events + replay_cursor.
```

Concurrency rule:
- execution concurrency must not change output ordering.

## 4) Outcome Mapping Rules

Given `AdmittedAction` and `EndpointExecutionOutcome`:

1. `Applied`
- `actual_cost_micro` from endpoint outcome.
- carries `reserve_entry_id`, `cost_attribution_id` from admitted action.

2. `Rejected`
- `reason_code` from endpoint outcome, else deterministic fallback.
- carries settlement linkage fields.

3. `Deferred`
- no settlement linkage today (preserves existing type).
- continuity behavior remains no-op for deferred.

4. endpoint invocation error (`Err`)
- mapped to `ActionRejected` with:
  - `reason_code = "endpoint_error"`
  - `reference_id = "spine:error:<action_id>"`

5. route miss
- mapped to `ActionRejected` with:
  - `reason_code = "route_not_found"`
  - `reference_id = "spine:missing_route:<action_id>"`

## 5) Replay Cursor Rule

Deterministic cursor format:
- `route:<cycle_id>:<event_count>:<version>`

This is opaque to consumers; only stability and monotonic traceability matter.

## 6) Catalog-to-Cortex Bridge Algorithm

Input: `SpineCapabilityCatalog`
Output: `cortex::CapabilityCatalog`

Pseudo-flow:
```text
1. group entries by affordance_key.
2. for each group:
   - sort and dedupe capability_handle list.
   - select first descriptor as representative (consistency guaranteed by registration invariant).
   - map default_cost -> RequestedResources field-for-field.
3. sort affordances by affordance_key.
4. emit Cortex catalog with version="spine:v<version>".
```

## 7) Complexity Targets

1. Route lookup: `O(log N)` via `BTreeMap`.
2. Registration: `O(log N + K)` where `K` entries share affordance key.
3. Serialized execution: `O(M log N)` for `M` actions.
4. Best-effort execution: `O(M log N)` lookup + async dispatch overhead + `O(M log M)` final order sort.

## 8) L2-03 Exit Criteria
This file is complete when:
1. routing/registration behavior is algorithmically explicit,
2. miss/error mapping is deterministic,
3. catalog bridge behavior is mechanically defined.
