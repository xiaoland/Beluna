# L2 Plan 02 - Domain Model And Interfaces
- Task Name: `core-cortex-act-stem-refactor`
- Stage: `L2`
- Focus: canonical contracts for Sense/State/Act and stage interfaces
- Status: `DRAFT_FOR_APPROVAL`

## 1) Canonical Shared Runtime Types
Create shared contracts in `core/src/runtime_types.rs` to break old `admission` coupling.

```rust
pub type SenseId = String;
pub type ActId = String;
pub type CycleId = u64;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequestedResources {
    pub survival_micro: i64,
    pub time_ms: u64,
    pub io_units: u64,
    pub token_units: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SenseDatum {
    pub sense_id: SenseId,
    pub source: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityPatch {
    pub entries: Vec<crate::spine::types::EndpointCapabilityDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityDropPatch {
    pub routes: Vec<crate::spine::types::RouteKey>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Sense {
    Domain(SenseDatum),
    Sleep,
    NewCapabilities(CapabilityPatch),
    DropCapabilities(CapabilityDropPatch),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoalFrame {
    pub goal_id: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CognitionState {
    pub revision: u64,
    pub goal_stack: Vec<GoalFrame>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhysicalLedgerSnapshot {
    pub available_survival_micro: i64,
    pub open_reservation_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhysicalState {
    pub cycle_id: CycleId,
    pub ledger: PhysicalLedgerSnapshot,
    pub capabilities: crate::cortex::CapabilityCatalog,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Act {
    pub act_id: ActId,
    pub based_on: Vec<SenseId>,
    pub endpoint_id: String,
    pub capability_id: String,
    pub capability_instance_id: String,
    pub normalized_payload: serde_json::Value,
    pub requested_resources: RequestedResources,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchDecision {
    Continue,
    Break,
}
```

## 2) Cortex Boundary Contract
Conceptual pure function contract is preserved. Implementation remains async-safe because AI Gateway is allowed.

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CortexOutput {
    pub acts: Vec<Act>,
    pub new_cognition_state: CognitionState,
}

#[async_trait]
pub trait CortexPort: Send + Sync {
    async fn cortex(
        &self,
        sense: &Sense,
        physical_state: &PhysicalState,
        cognition_state: &CognitionState,
    ) -> Result<CortexOutput, crate::cortex::CortexError>;
}
```

Contract rules:
1. Cortex cannot mutate external components.
2. Cortex must not persist cognition internally.
3. Same inputs + same model outputs + same clamp config => same `acts/new_cognition_state`.
4. Stem must never call Cortex for `Sense::Sleep`.

## 3) Ledger Stage Interface
Admission is gone; Ledger performs mechanical pre-dispatch reservation and post-spine settlement.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerDispatchTicket {
    pub reserve_entry_id: String,
    pub cost_attribution_id: String,
    pub reserved_survival_micro: i64,
}

#[derive(Debug, Clone)]
pub struct DispatchContext {
    pub cycle_id: CycleId,
    pub act_seq_no: u64,
}

pub trait LedgerStagePort: Send + Sync {
    fn pre_dispatch(
        &mut self,
        act: &Act,
        ctx: &DispatchContext,
    ) -> Result<(DispatchDecision, Option<LedgerDispatchTicket>), crate::continuity::ContinuityError>;

    fn settle_from_spine(
        &mut self,
        ticket: &LedgerDispatchTicket,
        event: &crate::spine::types::SpineEvent,
        ctx: &DispatchContext,
    ) -> Result<(), crate::continuity::ContinuityError>;

    fn physical_snapshot(&self) -> PhysicalLedgerSnapshot;
}
```

Determinism rule:
1. `cost_attribution_id` is derived in ledger stage from `(cycle_id, act_id)` deterministic hash.
2. `reserve_entry_id` assignment remains ledger-sequence deterministic.

## 4) Continuity Interface
Continuity persists cognition and capability overlays and participates in dispatch gating.

```rust
pub trait ContinuityStagePort: Send + Sync {
    fn cognition_state_snapshot(&self) -> CognitionState;
    fn persist_cognition_state(
        &mut self,
        state: CognitionState,
    ) -> Result<(), crate::continuity::ContinuityError>;

    fn apply_capability_patch(&mut self, patch: &CapabilityPatch);
    fn apply_capability_drop(&mut self, drop: &CapabilityDropPatch);

    fn capabilities_snapshot(&self) -> crate::cortex::CapabilityCatalog;

    fn pre_dispatch(
        &mut self,
        act: &Act,
        cognition_state: &CognitionState,
        ctx: &DispatchContext,
    ) -> Result<DispatchDecision, crate::continuity::ContinuityError>;

    fn on_spine_event(
        &mut self,
        act: &Act,
        event: &crate::spine::types::SpineEvent,
        ctx: &DispatchContext,
    ) -> Result<(), crate::continuity::ContinuityError>;
}
```

Patch/drop semantics:
1. arrival-order-wins,
2. drop applies tombstone by route key,
3. new patch removes tombstone for same route and upserts entry.

## 5) Spine Interface
Spine executes one act request at a time (Stem serial flow) but keeps ordered/replayable event semantics.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActDispatchRequest {
    pub cycle_id: CycleId,
    pub seq_no: u64,
    pub act: Act,
    pub reserve_entry_id: String,
    pub cost_attribution_id: String,
}

#[async_trait]
pub trait SpineStagePort: Send + Sync {
    async fn dispatch_act(
        &self,
        req: ActDispatchRequest,
    ) -> Result<crate::spine::types::SpineEvent, crate::spine::SpineError>;

    fn capability_catalog_snapshot(&self) -> crate::spine::types::SpineCapabilityCatalog;
}
```

Invariant retention:
1. Spine settlement events keep `reserve_entry_id` and `cost_attribution_id`.
2. `seq_no` remains total-order key per cycle in serial mode.

## 6) Physical State Composition Interface
Stem composes `PhysicalState` from three contributors and then invokes Cortex.

```rust
pub trait CapabilityContributor {
    fn capability_catalog_snapshot(&self) -> crate::cortex::CapabilityCatalog;
}

pub fn compose_physical_state(
    cycle_id: CycleId,
    ledger: PhysicalLedgerSnapshot,
    spine_caps: crate::cortex::CapabilityCatalog,
    continuity_caps: crate::cortex::CapabilityCatalog,
    ledger_caps: crate::cortex::CapabilityCatalog,
) -> PhysicalState;
```

Merge rules:
1. base catalog from Spine,
2. overlay Continuity,
3. overlay Ledger,
4. overlay continuity patch/tombstone transformations (already reflected in continuity snapshot).

## 7) L2-02 Exit Conditions
1. no type depends on removed Admission module,
2. `Act` is canonical execution intent unit,
3. stage contracts encode `Continue|Break` only,
4. interfaces are detailed enough for direct implementation in L3.

Status: `READY_FOR_REVIEW`
