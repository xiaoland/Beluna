# L2 Plan 02b - Effectuation, Ledger, And Continuity Model
- Task Name: `refactor-mind-cortex-non-cortex`
- Stage: `L2` / Part 02b
- Date: `2026-02-10`
## 1) Core Identifiers
```rust
pub type CycleId = u64;
pub type AttemptId = String;
pub type CostAttributionId = String;
pub type ActionId = String;
pub type ConstraintCode = String;
pub type AffordanceKey = String;
pub type CapabilityHandle = String;
pub type LedgerEntryId = u64;
pub type SpineRequestId = String;
```
## 2) Effectuation And Admission Types
```rust
pub struct AdmittedAction {
    pub action_id: ActionId,
    pub source_attempt_id: AttemptId,
    pub cost_attribution_id: CostAttributionId,
    pub reserve_entry_id: LedgerEntryId,
    pub capability_handle: CapabilityHandle,
    pub affordance_key: AffordanceKey,
    pub normalized_payload: serde_json::Value,
    pub degradation_profile_id: Option<String>,
    pub reserved_cost: CostVector,
    admission_proof: AdmissionProof,
}
struct AdmissionProof(());
```
Mechanical enforcement:
1. `admission_proof` is private to non-cortex.
2. only non-cortex resolver can construct `AdmittedAction`.
Deterministic `ActionId` derivation:
```text
action_id =
  "act:" + hex(sha256(canonical_json({
    cycle_id,
    source_attempt_id,
    reserve_entry_id
  })))[0..24]
```
```rust
pub enum EffectuationDisposition {
    Admitted {
        degraded: bool,
    },
    DeniedHard {
        code: ConstraintCode,
    },
    DeniedEconomic {
        code: ConstraintCode,
    },
}
pub struct CostVector {
    pub survival_micro: i128,
    pub time_ms: u64,
    pub io_units: u64,
    pub token_units: u64,
}
pub struct AffordabilitySnapshot {
    pub available_survival_micro: i128,
    pub required_survival_micro: i128,
    pub reserve_survival_micro: i128,
    pub policy_id: String,
}
```
```rust
pub enum AdmissionReportResult {
    Admitted {
        degraded: bool,
        reserve_entry_id: LedgerEntryId,
        reserve_amount_micro: i128,
        degradation_profile_id: Option<String>,
        affordability: AffordabilitySnapshot,
    },
    DeniedHard {
        code: ConstraintCode,
    },
    DeniedEconomic {
        code: ConstraintCode,
        affordability: AffordabilitySnapshot,
    },
}
pub struct EffectuationOutcome {
    pub attempt_id: AttemptId,
    pub cost_attribution_id: CostAttributionId,
    pub disposition: EffectuationDisposition,
    pub admitted_action_id: Option<ActionId>,
    pub report_result: AdmissionReportResult,
}
pub struct AdmissionReportItem {
    pub attempt_id: AttemptId,
    pub cost_attribution_id: CostAttributionId,
    pub result: AdmissionReportResult,
}
pub struct AdmissionReport {
    pub cycle_id: CycleId,
    pub items: Vec<AdmissionReportItem>,
}
```
## 3) Global Survival Ledger Types
```rust
pub enum LedgerDirection {
    Debit,
    Credit,
    Adjustment,
}
pub enum LedgerSource {
    AdmissionReserve,
    SpineSettlement,
    AIGatewayApprox,
    ManualGrant,
}
pub enum LedgerAccuracy {
    Exact,
    Approximate,
}
pub enum ReservationTerminal {
    Settled,
    Refunded,
    Expired,
}
pub struct ReservationState {
    pub reserve_entry_id: LedgerEntryId,
    pub amount_micro: i128,
    pub cost_attribution_id: CostAttributionId,
    pub created_cycle: CycleId,
    pub expires_at_cycle: CycleId,
    pub terminal: Option<ReservationTerminal>,
    pub terminal_reference_id: Option<String>,
}
pub struct SurvivalLedgerEntry {
    pub id: LedgerEntryId,
    pub cycle_id: CycleId,
    pub direction: LedgerDirection,
    pub amount_micro: i128,
    pub source: LedgerSource,
    pub accuracy: LedgerAccuracy,
    pub reference_id: String,
    pub note: String,
}
pub struct SurvivalLedger {
    pub balance_micro: i128,
    pub floor_micro: i128,
    pub next_entry_id: LedgerEntryId,
    pub entries: std::collections::VecDeque<SurvivalLedgerEntry>,
    pub reservations: std::collections::BTreeMap<LedgerEntryId, ReservationState>,
    pub seen_external_refs: std::collections::BTreeSet<String>,
}
```
## 4) Continuity Kernel State
```rust
pub struct NonCortexState {
    pub cycle_id: CycleId,
    pub continuity_id: String,
    pub affordance_registry_version: String,
    pub cost_policy_version: String,
    pub admission_ruleset_version: String,
    pub reservation_ttl_cycles: u64,
    pub ledger: SurvivalLedger,
    pub affordance_registry: std::collections::BTreeMap<AffordanceKey, AffordanceProfile>,
    pub last_outcomes: std::collections::VecDeque<EffectuationOutcome>,
}
```
Clock definition:
1. reservation timeout uses cycle clock only.
2. expiry condition: `current_cycle >= expires_at_cycle`.
3. no wall-clock expiration logic.
## 5) Spine Contract Types (Contracts Only)
```rust
pub struct AdmittedActionBatch {
    pub cycle_id: CycleId,
    pub request_id: SpineRequestId,
    pub actions: Vec<AdmittedAction>,
}
pub enum SpineExecutionMode {
    BestEffortReplayable,
    SerializedDeterministic,
}
pub enum SpineEvent {
    ActionStarted {
        action_id: ActionId,
        cost_attribution_id: CostAttributionId,
        reserve_entry_id: LedgerEntryId,
    },
    ActionApplied {
        action_id: ActionId,
        cost_attribution_id: CostAttributionId,
        reserve_entry_id: LedgerEntryId,
        actual_cost_micro: Option<i128>,
    },
    ActionRejected {
        action_id: ActionId,
        cost_attribution_id: CostAttributionId,
        reserve_entry_id: LedgerEntryId,
        reason_code: String,
    },
    SensorFeedback {
        key: String,
        payload: serde_json::Value,
    },
    BatchCompleted {
        request_id: SpineRequestId,
    },
}
pub struct OrderedSpineEvent {
    pub seq_no: u64,
    pub event: SpineEvent,
}
pub struct SpineExecutionReport {
    pub request_id: SpineRequestId,
    pub mode: SpineExecutionMode,
    pub events: Vec<OrderedSpineEvent>,
    pub replay_cursor: Option<String>,
}
```
## 6) Invariants (Effectuation/Ledger/Continuity)
1. Admission boundary:
- only `AdmittedAction[]` can be sent to spine.
- denied attempts never cross the spine boundary.
2. Denial reason completeness:
- `DeniedHard` and `DeniedEconomic` always carry a concrete `code`.
3. Admission purity:
- no wall-clock, randomness, or unordered iteration.
- decisions may depend only on:
  - `NonCortexState`
  - attempt (`affordance_key`, `capability_handle`, `requested_resources`, `normalized_payload`)
  - deterministic registries/policies and active version tuple (`affordance_registry_version`, `cost_policy_version`, `admission_ruleset_version`).
4. Hard-before-economic:
- hard impossibility checks execute before economic checks.
5. Ledger safety:
- admission debits cannot move balance below `floor_micro`.
6. Settlement consistency:
- every reservation reaches terminal state within bounded cycles: `Settled | Refunded | Expired`.
- settlement is idempotent by `(reserve_entry_id, reference_id)`.
7. Reservation terminal strictness:
- at most one terminal transition in `{settle, refund, expire}` per reservation.
8. Spine ordering semantics:
- `BestEffortReplayable`: nondeterministic execution allowed, but ordered replayable events required.
- `SerializedDeterministic`: deterministic serialized event order required.
9. Cost attribution chain:
- `cost_attribution_id` is carried through attempt -> admitted action -> spine event -> external debit.
- unmatched attribution is ignored for external debits.
10. External debit idempotency:
- duplicate external `reference_id` observations are ignored.
11. Versioned determinism:
- reproducibility/audit requires interpreting outcomes under the state's explicit version tuple.
