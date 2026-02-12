# L2-02 - Core Interfaces And Data Model
- Task Name: `spine-implementation`
- Stage: `L2` detailed file
- Date: `2026-02-12`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Canonical Semantics

1. `affordance_key`
- defines what operation shape exists.
- examples: `observe.state`, `execute.tool`, `notify.webhook`.

2. `capability_handle`
- defines which concrete execution channel realizes that affordance.
- examples: `cap.native.fs`, `cap.unix.local-agent`, `cap.websocket.remote`.

3. composite route key
- `RouteKey = (affordance_key, capability_handle)`.
- Spine routing is exact match only.

4. Body Endpoint -> Spine ingress datum
- named `Sense`.
- transport-agnostic semantic type carried by adapter wire and mapped to Cortex `SenseDelta`.

## 2) Core Data Structures (Spine-owned)

### 2.1 Route and capability registration
```rust
pub struct RouteKey {
    pub affordance_key: String,
    pub capability_handle: String,
}

pub struct EndpointCapabilityDescriptor {
    pub route: RouteKey,
    pub payload_schema: serde_json::Value,
    pub max_payload_bytes: usize,
    pub default_cost: CostVector,
    pub metadata: BTreeMap<String, String>,
}

pub struct EndpointRegistration {
    pub endpoint_id: String,
    pub descriptor: EndpointCapabilityDescriptor,
}
```

### 2.2 Spine catalog snapshot
```rust
pub struct SpineCapabilityCatalog {
    pub version: u64,
    pub entries: Vec<EndpointCapabilityDescriptor>,
}
```

Catalog entries are sorted deterministically by:
1. `affordance_key`
2. `capability_handle`

### 2.3 Endpoint invocation and outcome
```rust
pub struct EndpointInvocation {
    pub action: AdmittedAction,
}

pub enum EndpointExecutionOutcome {
    Applied {
        actual_cost_micro: i64,
        reference_id: String,
    },
    Rejected {
        reason_code: String,
        reference_id: String,
    },
    Deferred {
        reason_code: String,
    },
}
```

## 3) Async Port Interfaces

Use `async_trait` for object-safe async trait usage.

```rust
#[async_trait]
pub trait EndpointPort: Send + Sync {
    async fn invoke(
        &self,
        invocation: EndpointInvocation,
    ) -> Result<EndpointExecutionOutcome, SpineError>;
}

pub trait EndpointRegistryPort: Send + Sync {
    fn register(
        &self,
        registration: EndpointRegistration,
        endpoint: Arc<dyn EndpointPort>,
    ) -> Result<(), SpineError>;

    fn unregister(&self, route: &RouteKey) -> Option<EndpointRegistration>;

    fn resolve(&self, route: &RouteKey) -> Option<Arc<dyn EndpointPort>>;

    fn catalog_snapshot(&self) -> SpineCapabilityCatalog;
}

#[async_trait]
pub trait SpineExecutorPort: Send + Sync {
    fn mode(&self) -> SpineExecutionMode;

    async fn execute_admitted(
        &self,
        admitted: AdmittedActionBatch,
    ) -> Result<SpineExecutionReport, SpineError>;

    fn capability_catalog_snapshot(&self) -> SpineCapabilityCatalog;
}
```

## 4) Continuity Boundary Updates

Continuity spine call path becomes async:

```rust
#[async_trait]
pub trait SpinePort: Send + Sync {
    async fn execute_admitted(
        &self,
        admitted: AdmittedActionBatch,
    ) -> Result<SpineExecutionReport, ContinuityError>;
}
```

Engine methods update:
1. `ContinuityEngine::effectuate_attempts` -> `async fn`
2. `ContinuityEngine::process_attempts` -> `async fn`

## 5) Catalog Bridge To Cortex

Spine owns registration catalog, but Cortex currently consumes `cortex::CapabilityCatalog`.

Bridge rule in adapter/runtime layer:
1. group Spine entries by `affordance_key`.
2. build `allowed_capability_handles` from grouped entries.
3. copy schema/limits/default resources from representative descriptor.
4. set version string from Spine catalog version, e.g. `spine:v42`.

Invariant enforced at registration time:
- For the same `affordance_key`, descriptors must share:
  1. `payload_schema`
  2. `max_payload_bytes`
  3. `default_cost`

If not, registration is rejected as invariant violation.

## 6) Error Model Extensions

Extend `SpineErrorKind` with deterministic routing/registry classes:
1. `RouteConflict`
2. `RouteNotFound`
3. `RegistrationInvalid`

Core rule:
- lookup miss during execution is not a batch-level error;
- it becomes per-action `ActionRejected` event with deterministic `reason_code`.

## 7) Registration Scope (This Task)

1. in-scope:
- in-process registration API on Spine registry/executor.
- runtime bootstrap registration at startup.
- runtime-time registration via direct Rust API calls.

2. out-of-scope:
- remote self-registration protocol over UnixSocket/WebSocket wire messages.
- distributed registration auth/lease/heartbeat semantics.

## 8) L2-02 Exit Criteria
This file is complete when:
1. async trait signatures are implementation-ready,
2. data models cover registration/catalog/routing/endpoint outcomes,
3. catalog bridge to Cortex contract is explicit.
