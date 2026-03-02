# L2 Plan 02 - Sense Model and Wire Migration
- Task: `cortex-loop-architecture`
- Micro-task: `02-sense-model-and-wire-migration`
- Stage: `L2`
- Date: `2026-03-01`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Goal and Scope
Goal:
1. Hard-cut `Sense` to a single domain struct (old `SenseDatum` becomes `Sense`).
2. Migrate payload to strict text and remove `metadata` field.
3. Remove control-sense variants (`New/Drop NeuralSignalDescriptors`, `New/Drop Proprioceptions`, `Hibernate`).
4. Route descriptor/proprioception changes through Spine runtime control path; Spine runtime then calls Stem directly.

In scope:
1. `core` type and runtime model changes.
2. `spine` adapter ingress validation and dispatch changes.
3. `cli` and `apple-universal` wire contract migration.
4. deterministic metadata rendering for Cortex Primary input lines.

Out of scope:
1. Afferent deferral policy implementation (`03`).
2. Structured act tool schema migration (`04`).
3. Full docs/contract sweep (`08` owns final authoritative refresh).

## 2) Canonical Sense Contract
`Sense` replaces enum and old `SenseDatum`.

```rust
pub struct Sense {
    pub sense_instance_id: String,
    pub endpoint_id: String,
    pub neural_signal_descriptor_id: String,
    pub payload: String,
    pub weight: f64,                  // default 0.0, must be within [0.0, 1.0]
    pub act_instance_id: Option<String>,
}
```

Removed from `Sense` model:
1. `metadata`
2. enum wrapping (`Domain`)
3. `Hibernate`
4. `NewNeuralSignalDescriptors`, `DropNeuralSignalDescriptors`
5. `NewProprioceptions`, `DropProprioceptions`

## 3) Spine-Mediated Control Contract
Adapter-to-Stem direct calls are not allowed. Control mutations are two-hop:
1. adapters call Spine runtime control API.
2. Spine runtime updates Spine-owned state and then calls Stem control API directly.

Adapter-facing port (implemented by Spine runtime):

```rust
#[async_trait]
pub trait SpineControlPort: Send + Sync {
    async fn register_neural_signal_descriptors(&self, endpoint_id: &str, entries: Vec<NeuralSignalDescriptor>);
    async fn drop_neural_signal_descriptors(&self, endpoint_id: &str, routes: Vec<NeuralSignalDescriptorRouteKey>);
    async fn patch_endpoint_proprioceptions(&self, endpoint_id: &str, entries: BTreeMap<String, String>);
    async fn drop_endpoint_proprioceptions(&self, endpoint_id: &str, keys: Vec<String>);
}
```

Spine-to-Stem control port:

```rust
#[async_trait]
pub trait StemControlPort: Send + Sync {
    async fn apply_neural_signal_descriptor_patch(&self, patch: NeuralSignalDescriptorPatch);
    async fn apply_neural_signal_descriptor_drop(&self, drop_patch: NeuralSignalDescriptorDropPatch);
    async fn apply_proprioception_patch(&self, patch: ProprioceptionPatch);
    async fn apply_proprioception_drop(&self, drop_patch: ProprioceptionDropPatch);
}
```

Ownership:
1. Stem owns physical-state neural-signal descriptor catalog and proprioception state.
2. Spine runtime owns Spine endpoint catalog/proprioception mirrors and acts as adapter control gateway.
3. Spine adapters can only call `SpineControlPort`.
4. Spine runtime calls `StemControlPort` to sync control mutations into Stem-owned physical state.
5. No control mutation is encoded as synthetic senses anymore.

## 4) State Ownership Impact
1. Stem owns neural signal descriptor catalog for physical state.
2. Stem owns shared proprioception state for physical state.
3. Spine runtime owns adapter-facing endpoint catalog state and forwards mutations to Stem.
4. `PhysicalStateReadPort` snapshots include latest descriptor + proprioception states from Stem.

## 5) Afferent Pathway Contract After Migration
1. Afferent queue carries only `Sense` domain payloads.
2. `send_hibernate_blocking()` is removed.
3. Shutdown is lifecycle/cancellation based only (no control sense injection).

## 6) Ingress/Wire Validation Rules
For inbound `sense` messages (unix-socket and inline materialization):
1. `payload` must be non-empty text.
2. `weight` absent => default `0.0`; present must satisfy `[0.0, 1.0]`.
3. `act_instance_id` absent is allowed; present must pass configured UUID validation.
4. `metadata` field is rejected as unknown/invalid.

## 7) Cortex Rendering Contract
Senses delivered to Primary are rendered as:

```text
- [fq-sense-id]. [key=value,key=value,...]; [payload-truncated-if-needed]
```

Metadata rendering source:
1. deterministic derivation from typed Sense fields excluding `sense_instance_id`, `endpoint_id`, `neural_signal_descriptor_id`, and `payload`.
2. stable key ordering and escaping are mandatory.

## 8) File/Interface Freeze for L3
1. `core/src/types.rs`
- replace Sense enum with Sense struct.
2. `core/src/afferent_pathway.rs`
- queue type remains, but `Hibernate`-specific API removed.
3. `core/src/stem/runtime.rs`
- define/host `StemControlPort` implementation and shared control state hooks.
4. `core/src/cortex/runtime.rs`, `core/src/cortex/primary.rs`, helper modules
- consume `Vec<Sense>` directly (no `Sense::Domain` filtering).
5. `core/src/spine/runtime.rs`
- define/host `SpineControlPort` implementation and call `StemControlPort`.
6. `core/src/spine/adapters/inline.rs`, `core/src/spine/adapters/unix_socket.rs`
- replace synthetic control-sense emits with `SpineControlPort` calls only.
7. `cli/src/main.rs`, `apple-universal/.../BodyEndpointWire.swift`
- sense wire updated to text payload + weight + optional `act_instance_id`.

## 9) Risks and Constraints
1. Direct call path can over-couple adapters to Stem if raw concrete runtime types are passed.
Mitigation: adapters are restricted to `SpineControlPort`; only Spine runtime can call `StemControlPort`.
2. Hard-cut schema can break all endpoint emitters at once.
Mitigation: compile-break migration with explicit call-site checklist and no compatibility window.
3. Removing enum variants can leave stale matching logic.
Mitigation: enforce grep/build gates in L3.

## 10) L2 Exit Criteria (02)
1. `Sense` canonical schema and validation are frozen.
2. Control updates are frozen to Spine-runtime-mediated direct Stem calls, not sense events.
3. Afferent queue contract is domain-sense-only.
4. File/interface impact map is complete and implementation-ready.

Status: `READY_FOR_REVIEW`
