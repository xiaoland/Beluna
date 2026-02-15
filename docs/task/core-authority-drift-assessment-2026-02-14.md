# Core Drift Assessment against Authority Statements (2026-02-14)

## Scope

This note compares current `core/` behavior with the provided authority statements and records where drift exists.

## Drift Matrix (Revised)

### 1) Capability payload schema and Act payload JSON

- **Authority**: Capability has payload schema; Act payload is JSON.
- **Current**: `EndpointCapabilityDescriptor.payload_schema` exists; `Act.normalized_payload` is `serde_json::Value`.
- **Assessment**: **No drift**.

### 2) Sense / Act / Capability declaration ownership

- **Authority**: Sense, Act, Capability are all declared under Cortex ownership semantics.
- **Current**: Sense/Act are in `runtime_types`; capability descriptor and routing key are primarily in Spine types; Cortex consumes a projection.
- **Assessment**: **Drift** (ownership split).

### 3) Endpoint identity model (id vs name)

- **Authority (revised)**:
  - `body-endpoint-id` and `body-endpoint-name` are separate.
  - `body-endpoint-id` is UUID assigned by Spine.
  - body endpoint provides semantic endpoint name.
  - Spine appends a monotonic numeric suffix to produce the fully-qualified endpoint name at **endpoint registration time** (example: `cli.1`, `macos-app.2`).
- **Current**:
  - No first-class endpoint-name field in registry route model.
  - Endpoint id currently comes from registering side data (not Spine-issued UUID).
  - No monotonic suffix assignment at endpoint registration.
- **Assessment**: **Major drift**.

### 4) Capability scope and routing semantics

- **Authority (revised)**:
  - Capability scope is within a body endpoint.
  - No need to enforce capability singleton in catalog because each Act carries fully-qualified endpoint name, and capability routing is completed inside the body endpoint.
- **Current**:
  - Registry route key is `(endpoint_id, capability_id)` and resolves endpoint by `act.endpoint_id`.
  - System still models capability routing centrally at Spine route layer.
- **Assessment**: **Drift** (routing locus differs from target semantics).

### 5) Registration/control flow boundaries

- **Authority (revised)**:
  - Body endpoint registration is **not** a sense.
  - Authentication can carry capabilities declaration; adapter transforms auth payload (including optional capabilities) and calls Spine `new_body_endpoint`.
  - Endpoint unplug request is sent by body endpoint; adapter then calls Spine `remove_body_endpoint`.
  - Only capability changes go through sense flow.
- **Current**:
  - Adapter currently accepts wire messages for body endpoint register/unregister and separately emits capability patch senses.
  - Control boundaries are mixed; registration is coupled with wire ingress message parsing format instead of explicit post-auth API call contract.
- **Assessment**: **Major drift**.

### 6) Adapter role, session ownership, and lifecycle

- **Authority (revised)**:
  - Adapter is Spine API transport conversion layer.
  - Adapter manages endpoint sessions/connections; Spine does **not** manage body endpoint session internals and only tracks endpoint-to-adapter ownership mapping.
  - Adapter identity is Spine-assigned incremental integer sequence (`adapter_id`) at Spine startup.
  - Spine `start` should start all adapters from `config.spine.adapters[]`, where each entry is `{ type, config }` (e.g. `unix-socket-ndjson` + `socket_path`).
- **Current**:
  - Unix adapter is a monolithic `run` loop and endpoint session state is mixed into Spine registry internals.
  - Adapter boot is hardcoded in `main` as a single unix adapter instead of adapter-array-driven Spine startup.
  - No explicit Spine-assigned incremental `adapter_id` model exists.
- **Assessment**: **Major drift**.

### 7) NDJSON over Unix Socket frame shape

- **Authority**: Envelope fields are `method`, `id` (uuidv4), `timestamp` (UTC ms), `body`; inbound body methods are `sense`/`auth`; adapter outbound method is `act`.
- **Current**: Tagged `type` model is used; envelope lacks required `id` + `timestamp`; `auth` method absent.
- **Assessment**: **Major drift**.

### 8) Auth payload capability declaration handling

- **Authority (revised)**: Body endpoint authentication may include capabilities declaration; `new_body_endpoint` can consume it directly.
- **Current**: Capability declarations are primarily modeled as separate registration/capability messages, not auth-coupled endpoint bootstrap input.
- **Assessment**: **Drift**.

### 9) In-core body endpoint startup sequence

- **Authority**: In-core body endpoints start conditionally in `main`, after Spine starts and before Stem starts.
- **Current**: Endpoint registration currently occurs before spine singleton installation.
- **Assessment**: **Minor drift**.

## Planning Location

Detailed remediation steps are maintained in:

- `docs/task/core-authority-drift-remediation-plan-2026-02-14.md`
