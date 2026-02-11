# L2 Plan 03 - Ports And Policy Contracts
- Task Name: `refactor-mind-cortex-non-cortex`
- Stage: `L2` / Part 03
- Date: `2026-02-10`
## 1) Cortex -> Non-Cortex Admission Port
```rust
pub trait AdmissionResolverPort: Send + Sync {
    fn resolve_attempts(
        &mut self,
        cycle_id: CycleId,
        attempts: &[IntentAttempt],
    ) -> Result<AdmissionBatchResult, NonCortexError>;
}
```
```rust
pub struct AdmissionBatchResult {
    pub outcomes: Vec<EffectuationOutcome>,
    pub admitted_actions: Vec<AdmittedAction>,
    pub admission_report: AdmissionReport,
}
```
Contract requirements:
1. `outcomes.len() == attempts.len()`.
2. order is deterministic by sorted `attempt_id`.
3. `admitted_actions` includes only outcomes with `EffectuationDisposition::Admitted { .. }`.
4. `admission_report` includes every attempt (including denied ones) and is returned to cortex each cycle.
5. no semantic interpretation of intent text.
## 2) Non-Cortex -> Spine Execution Port
```rust
pub trait SpinePort: Send + Sync {
    fn execute_admitted(
        &self,
        batch: &AdmittedActionBatch,
    ) -> Result<SpineExecutionReport, SpineError>;
}
```
Mechanical guardrail:
1. method accepts `AdmittedActionBatch` only.
2. there is no method overload for `IntentAttempt`.
3. spine must declare/emit semantics via `SpineExecutionReport.mode`:
- `BestEffortReplayable` (nondeterministic execution allowed, but ordered replayable events required)
- `SerializedDeterministic` (deterministic serialized execution order)
## 3) External Debit Source Port (Global Ledger Feed)
```rust
pub trait ExternalDebitSourcePort: Send + Sync {
    fn drain_observations(
        &mut self,
        after_cursor: Option<String>,
    ) -> Result<Vec<ExternalDebitObservation>, NonCortexError>;
}
pub struct ExternalDebitObservation {
    pub source: LedgerSource,
    pub reference_id: String,
    pub cost_attribution_id: CostAttributionId,
    pub action_id: Option<ActionId>,
    pub cycle_id: Option<CycleId>,
    pub amount_micro: i128,
    pub accuracy: LedgerAccuracy,
    pub note: String,
    pub cursor: String,
}
```
Initial adapter:
- `AIGatewayApproxDebitSource` maps gateway telemetry into approximate debit observations.
Matching rule:
1. unmatched `cost_attribution_id` observations are ignored.
2. when present, `action_id` and `cycle_id` must match the attribution chain.
3. matched observations are deduped by `reference_id`.
## 4) Affordance Policy Contracts
### 4.1 Hard constraint checker
```rust
pub trait HardConstraintPolicy: Send + Sync {
    fn check_hard(
        &self,
        state: &NonCortexState,
        attempt: &IntentAttempt,
        profile: &AffordanceProfile,
    ) -> Result<HardConstraintResult, NonCortexError>;
}
pub enum HardConstraintResult {
    Pass,
    Deny { code: ConstraintCode },
}
```
### 4.2 Economic estimator
```rust
pub trait EconomicPolicy: Send + Sync {
    fn estimate_cost(
        &self,
        state: &NonCortexState,
        attempt: &IntentAttempt,
        profile: &AffordanceProfile,
    ) -> Result<CostVector, NonCortexError>;
}
```
### 4.2b Cost admission policy (cross-dimension decision hook)
```rust
pub trait CostAdmissionPolicy: Send + Sync {
    fn affordability(
        &self,
        state: &NonCortexState,
        estimated: &CostVector,
        attempt: &IntentAttempt,
    ) -> Result<AffordabilitySnapshot, NonCortexError>;
    fn reserve_amount_micro(
        &self,
        snapshot: &AffordabilitySnapshot,
    ) -> Result<i128, NonCortexError>;
}
```
Contract requirements:
1. policy defines deterministic comparison/reservation across `survival_micro/time_ms/io_units/token_units`.
2. no implicit dimension comparison outside this policy.
### 4.3 Degradation policy
```rust
pub trait DegradationPolicy: Send + Sync {
    fn candidates(
        &self,
        state: &NonCortexState,
        attempt: &IntentAttempt,
        profile: &AffordanceProfile,
    ) -> Result<Vec<DegradationPlan>, NonCortexError>;
}
```
```rust
pub struct DegradationSearchPolicy {
    pub max_variants: usize,
    pub max_depth: usize,
    pub prefer_less_capability_loss: bool,
}
```
Deterministic rule:
1. candidate rank uses tuple:
- if `prefer_less_capability_loss=true`: `(capability_loss_score, estimated_survival_micro, profile_id)`
- else: `(estimated_survival_micro, capability_loss_score, profile_id)`
2. stop policy:
- search stops at first affordable candidate in ranked order,
- but never exceeds `max_variants` and `max_depth`.
## 5) Ledger Contract
```rust
impl SurvivalLedger {
    pub fn can_afford(&self, amount_micro: i128) -> bool;
    pub fn reserve(
        &mut self,
        cycle_id: CycleId,
        amount_micro: i128,
        cost_attribution_id: CostAttributionId,
        ttl_cycles: u64,
        reference_id: String,
    ) -> Result<LedgerEntryId, NonCortexError>;
    pub fn apply_entry(
        &mut self,
        cycle_id: CycleId,
        direction: LedgerDirection,
        amount_micro: i128,
        source: LedgerSource,
        accuracy: LedgerAccuracy,
        reference_id: String,
        note: String,
    ) -> LedgerEntryId;
    pub fn settle_reservation(
        &mut self,
        reserve_entry_id: LedgerEntryId,
        reference_id: String,
    ) -> Result<(), NonCortexError>;
    pub fn refund_reservation(
        &mut self,
        reserve_entry_id: LedgerEntryId,
        reference_id: String,
    ) -> Result<(), NonCortexError>;
    pub fn expire_reservation(
        &mut self,
        reserve_entry_id: LedgerEntryId,
        reference_id: String,
    ) -> Result<(), NonCortexError>;
}
```
Contract requirements:
1. deterministic entry IDs (`next_entry_id` monotonic).
2. no floating-point arithmetic in balance updates.
3. `seen_external_refs` prevents duplicate external debit ingestion.
4. strict reservation terminal invariant:
- at most one of `settle|refund|expire` per `reserve_entry_id`.
5. settlement/refund/expire are idempotent by `(reserve_entry_id, reference_id)`.
## 6) Cortex Planner Contract
```rust
pub trait AttemptPlanner: Send + Sync {
    fn plan_attempts(
        &self,
        state: &CortexState,
        scheduling: &[SchedulingContext],
        command: &CortexCommand,
    ) -> Result<Vec<IntentAttempt>, CortexError>;
}
```
Contract requirements:
1. deterministic for same state/command.
2. scheduling pressure is cycle-local input, not persisted into goal identity.
3. generates attempts without directly dispatching execution.
4. attempt IDs must follow the fixed canonical derivation contract in L2-02a.
## 6.1 Commitment manager contract
```rust
pub struct CommitmentManager;
impl CommitmentManager {
    pub fn register_goal(... ) -> Result<(), CortexError>;
    pub fn propose_commitment(... ) -> Result<CommitmentId, CortexError>;
    pub fn activate_commitment(... ) -> Result<(), CortexError>;
    pub fn transition_commitment(... ) -> Result<(), CortexError>;
    pub fn supersede_commitment(... ) -> Result<(), CortexError>;
}
```
## 7) Non-Cortex Facade Contract
```rust
pub struct NonCortexCycleOutput {
    pub outcomes: Vec<EffectuationOutcome>,
    pub admission_report: AdmissionReport,
    pub spine_report: SpineExecutionReport,
    pub external_ledger_entry_ids: Vec<LedgerEntryId>,
}
pub trait NonCortexKernelPort: Send + Sync {
    fn process_attempts(
        &mut self,
        cycle_id: CycleId,
        attempts: &[IntentAttempt],
        spine: &dyn SpinePort,
    ) -> Result<NonCortexCycleOutput, NonCortexError>;
}
```
## 8) Non-Interpretation Compliance Contract
Admission resolver must satisfy:
1. if two attempts differ only in semantic/narrative fields not used by affordance/cost tables, admission result must be identical.
2. branching input set is limited to deterministic fields (`affordance_key`, `capability_handle`, `requested_resources`, `normalized_payload`) plus deterministic non-cortex state/policies.
3. deterministic non-cortex state includes explicit version tuple:
- `affordance_registry_version`
- `cost_policy_version`
- `admission_ruleset_version`
4. denied outcomes use mechanical constraint codes only.
5. no policy language like "forbidden intent" in outcome rationale.
6. `AdmissionReportResult` payload is schema-limited (codes, affordability numbers, reserve deltas only).
## 9) No-op Adapters (For Deterministic Tests)
1. `NoopSpinePort`: returns `BatchCompleted` without side effects.
2. `NoopExternalDebitSource`: returns empty observation list.
3. `NoopAttemptPlanner`: returns empty attempt list.
