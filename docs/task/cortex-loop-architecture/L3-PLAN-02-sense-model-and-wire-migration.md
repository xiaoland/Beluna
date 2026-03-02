# L3 Plan 02 - Sense Model and Wire Migration
- Task: `cortex-loop-architecture`
- Micro-task: `02-sense-model-and-wire-migration`
- Stage: `L3`
- Date: `2026-03-01`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Objective
Implement a hard-cut migration to the new `Sense` contract and remove control-sense variants by moving descriptor/proprioception mutations to Spine-runtime-mediated direct Stem control calls.

## 2) Execution Steps
### Step 1 - Core Type Hard Cut
1. Update `core/src/types.rs`:
- delete Sense enum and legacy control variants.
- rename/replace old `SenseDatum` with canonical `Sense` struct.
- add fields: `payload: String`, `weight: f64`, `act_instance_id: Option<String>`.
- remove `metadata` and related defaults.
2. Keep patch/drop structs (`NeuralSignalDescriptorPatch`, `ProprioceptionPatch`, etc.) as control-port payload types.

Pseudo-code:
```rust
pub struct Sense {
    pub sense_instance_id: String,
    pub endpoint_id: String,
    pub neural_signal_descriptor_id: String,
    pub payload: String,
    pub weight: f64,
    pub act_instance_id: Option<String>,
}
```

### Step 2 - Afferent Contract Cleanup
1. Update `core/src/afferent_pathway.rs` to domain-sense-only semantics.
2. Remove `send_hibernate_blocking()` and any hibernate-only helpers.
3. Keep ingress/consumer/control handles introduced in micro-task `01`.

### Step 3 - Introduce Spine Adapter Control Port
1. Add `SpineControlPort` in `core/src/spine/runtime.rs`.
2. Implement control methods in Spine runtime for adapter use:
- update Spine-owned endpoint neural signal descriptor catalog state.
- update Spine-owned endpoint proprioception state.
3. Inject `Arc<dyn SpineControlPort>` into inline/unix-socket adapters.

### Step 4 - Introduce Stem Control Port (Spine -> Stem)
1. Add `StemControlPort` in `core/src/stem/runtime.rs`.
2. Implement Stem-owned physical-state mutation handlers:
- neural signal descriptor patch/drop on Stem-owned catalog.
- proprioception patch/drop on Stem-owned state.
3. Inject `Arc<dyn StemControlPort>` into Spine runtime only.
4. Spine runtime calls `StemControlPort` after updating its own state, so physical-state snapshots remain authoritative in Stem.

Pseudo-code:
```rust
#[async_trait]
trait StemControlPort {
    async fn apply_neural_signal_descriptor_patch(&self, patch: NeuralSignalDescriptorPatch);
    async fn apply_neural_signal_descriptor_drop(&self, drop_patch: NeuralSignalDescriptorDropPatch);
    async fn apply_proprioception_patch(&self, patch: ProprioceptionPatch);
    async fn apply_proprioception_drop(&self, drop_patch: ProprioceptionDropPatch);
}
```

### Step 5 - Replace Synthetic Control-Sense Emission
1. `core/src/spine/adapters/inline.rs`:
- replace `Sense::New/Drop*` sends with `spine_control.*` calls.
- keep only domain `Sense` send to afferent for body senses.
2. `core/src/spine/adapters/unix_socket.rs`:
- same replacement for auth/unplug/proprioception updates.
3. `core/src/stem/runtime.rs` efferent status updates:
- replace control-sense patch/drop calls with `StemControlPort` calls (inside Stem/efferent path only).

### Step 6 - Ingress Parse/Validation Migration
1. Update unix-socket inbound body schema for `sense`:
- payload text string
- optional `weight`
- optional `act_instance_id`
- reject `metadata`.
2. Update inline adapter `InlineSenseDatum` shape to text payload + optional weight/act correlation.
3. Enforce deterministic validation errors for invalid weight or act_instance_id.

### Step 7 - Cortex Consumption Path Migration
1. Update `core/src/cortex/runtime.rs`:
- remove enum matching (`Sense::Domain`) and consume `Sense` directly.
2. Update `core/src/cortex/primary.rs` and helpers:
- remove `Sense::Hibernate` guards.
- adapt helper input assumptions to struct-only `Sense`.

### Step 8 - Producer Wire Migration (`cli` and `apple-universal`)
1. `cli/src/main.rs`:
- emit new sense wire shape with text payload and optional weight/act_instance_id.
2. `apple-universal/.../BodyEndpointWire.swift`:
- remove `metadata` field from sense body.
- encode `act_instance_id` as first-class field.
- keep payload as text.

### Step 9 - Deterministic Rendering Contract Hook
1. Add/adjust shared formatter used by cortex sense input helper:
- deterministic key order
- deterministic escaping
- payload truncation by byte limit.
2. Ensure rendered metadata excludes `sense_instance_id`, `endpoint_id`, `neural_signal_descriptor_id`, `payload`.

## 3) File-Level Change Map
1. `core/src/types.rs`
2. `core/src/afferent_pathway.rs`
3. `core/src/stem/runtime.rs`
4. `core/src/spine/runtime.rs` (add `SpineControlPort`, call `StemControlPort`)
5. `core/src/main.rs` (control-port composition + shared proprioception state wiring)
6. `core/src/spine/adapters/inline.rs`
7. `core/src/spine/adapters/unix_socket.rs`
8. `core/src/cortex/runtime.rs`
9. `core/src/cortex/primary.rs`
10. `core/src/cortex/helpers/sense_input_helper.rs`
11. `cli/src/main.rs`
12. `apple-universal/BelunaApp/BodyEndpoint/BodyEndpointWire.swift`

## 4) Verification Gates
### Gate A - Contract/Enum Removal
```bash
rg -n "enum Sense|Sense::Domain|Sense::Hibernate|Sense::NewNeuralSignalDescriptors|Sense::DropNeuralSignalDescriptors|Sense::NewProprioceptions|Sense::DropProprioceptions" core/src
```
Expected: no remaining control-sense enum paths.

### Gate B - Metadata Removal
```bash
rg -n "SenseDatum\.metadata|metadata" core/src/spine/adapters/unix_socket.rs core/src/spine/adapters/inline.rs cli/src/main.rs apple-universal/BelunaApp/BodyEndpoint/BodyEndpointWire.swift
```
Expected: no sense-metadata field usage.

### Gate C - Control Path Layering
```bash
rg -n "StemControlPort|apply_neural_signal_descriptor_|apply_proprioception_" core/src/spine/runtime.rs core/src/stem/runtime.rs
rg -n "StemControlPort|stem::runtime" core/src/spine/adapters
```
Expected:
1. control mutations route Spine adapter -> Spine runtime -> Stem control port.
2. no adapter directly imports/calls Stem control types.

### Gate D - Build
Per workspace rule, build only:
```bash
cd core && cargo build
cd ../cli && cargo build
```

## 5) Completion Criteria (02)
1. Canonical `Sense` struct contract is fully migrated.
2. Afferent queue carries only domain senses.
3. Descriptor/proprioception updates occur via Spine-runtime-mediated direct Stem control calls.
4. CLI and Apple wire models align with text payload + weight + optional `act_instance_id`.
5. Core and CLI builds pass.

Status: `READY_FOR_REVIEW`
