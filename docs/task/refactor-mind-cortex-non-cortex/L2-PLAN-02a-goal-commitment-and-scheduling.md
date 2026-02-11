# L2 Plan 02a - Goal, Commitment, And Scheduling Model
- Task Name: `refactor-mind-cortex-non-cortex`
- Stage: `L2` / Part 02a
- Date: `2026-02-10`

## 1) Core Identifiers

```rust
pub type GoalId = String;
pub type CommitmentId = String;
pub type CycleId = u64;
pub type AttemptId = String;
pub type CostAttributionId = String;
pub type AffordanceKey = String;
pub type CapabilityHandle = String;
```

## 2) Axis Separation (Locked)

Goal representation is explicitly split into three axes:
1. Semantic identity axis (Cortex-owned): what the goal means.
2. Commitment/lifecycle axis (Cortex-owned): what is currently pursued.
3. Normative/scheduling axis (cycle-local, runtime/economic-coupled): what gets effort now under current external pressure.

This prevents runtime pressure from corrupting semantic goal identity.

## 3) Semantic Goal Model (No Importance/Power Fields)

```rust
pub enum GoalClass {
    Strategic,
    Project,
    Task,
    Custom(String),
}

pub struct Goal {
    pub id: GoalId,
    pub title: String,
    pub class: GoalClass,
    pub parent_goal_id: Option<GoalId>,
    pub spec: serde_json::Value,
    pub metadata: GoalMetadata,
}
```

Removed from `Goal` by design:
1. `GoalLevel` (importance proxy).
2. static `priority`.
3. `created_cycle`.

## 4) Typed Metadata With Provenance

```rust
pub type GoalMetadata = std::collections::BTreeMap<String, MetadataEntry>;

pub struct MetadataEntry {
    pub value: MetadataValue,
    pub provenance: MetadataProvenance,
    pub observed_cycle: CycleId,
}

pub enum MetadataValue {
    Text(String),
    Number(f64),
    Boolean(bool),
    Json(serde_json::Value),
}

pub enum MetadataProvenance {
    UserInput,
    CortexInference,
    RuntimeFeedback,
    ExternalTool { name: String },
}
```

## 5) Commitment Model (Goal != Commitment)

```rust
pub enum CommitmentStatus {
    Proposed,
    Active,
    Paused,
    Cancelled,
    Completed,
    Failed,
}

pub struct GoalCommitment {
    pub id: CommitmentId,
    pub goal_id: GoalId,
    pub status: CommitmentStatus,
    pub created_cycle: CycleId,
    pub last_transition_cycle: CycleId,
    pub superseded_by_goal_id: Option<GoalId>,
    pub failure_code: Option<String>,
}
```

Rules:
1. `Merged` is not a status.
2. merge/supersession is a relationship (`superseded_by_goal_id`).
3. `created_cycle` belongs to commitment, not semantic goal.

## 6) Dynamic Scheduling Context (Priority Is Computed)

```rust
pub struct SchedulingContext {
    pub cycle_id: CycleId,
    pub commitment_id: CommitmentId,
    pub computed_priority: f32,
    pub budget_pressure: f32,
    pub reliability_pressure: f32,
    pub external_urgency: f32,
}
```

Rules:
1. recomputed each cycle.
2. never persisted into semantic goal identity.

## 7) Non-Binding Intent Attempt Model

```rust
pub struct IntentAttempt {
    pub attempt_id: AttemptId,
    pub cost_attribution_id: CostAttributionId,
    pub commitment_id: Option<CommitmentId>,
    pub goal_id: Option<GoalId>,
    pub affordance_key: AffordanceKey,
    pub capability_handle: CapabilityHandle,
    pub normalized_payload: serde_json::Value,
    pub requested_resources: RequestedResources,
    pub metadata: std::collections::BTreeMap<String, serde_json::Value>,
}

pub struct RequestedResources {
    pub max_time_ms: Option<u64>,
    pub max_output_tokens: Option<u64>,
    pub io_units: Option<u64>,
}
```

Contract:
1. `IntentAttempt` is declarative and non-executable.
2. attempts can only become executable through non-cortex admission.

## 8) Deterministic AttemptId Derivation

`AttemptId` generation is fixed:

```text
attempt_id =
  "att:" + hex(sha256(canonical_json({
    cycle_id,
    commitment_id,
    goal_id,
    planner_slot,
    affordance_key,
    capability_handle,
    normalized_payload,
    requested_resources,
    cost_attribution_id
  })))[0..24]
```

Rules:
1. `canonical_json` uses stable key ordering and stable numeric formatting.
2. `planner_slot` is deterministic ordering index from planner output before ID assignment.
3. no random UUID/time-based ID generation is allowed.

## 9) Cortex State Container

```rust
pub struct CortexState {
    pub cycle_id: CycleId,
    pub goals: std::collections::BTreeMap<GoalId, Goal>,
    pub commitments: std::collections::BTreeMap<CommitmentId, GoalCommitment>,
    pub active_commitment_id: Option<CommitmentId>,
    pub pending_attempts: std::collections::VecDeque<IntentAttempt>,
}
```

## 10) Invariants (Goal/Commitment/Scheduling)

1. Goal/commitment separation:
- `Goal` stores semantic identity only.
- lifecycle data exists only on `GoalCommitment`.

2. Status model:
- commitment statuses include `Failed`.
- supersession uses relationship field, not status.

3. Dynamic scheduling:
- `computed_priority` is cycle-local input to planning.
- no static priority field in `Goal`.

4. Non-binding attempts:
- attempts are never directly executable.

5. Cost attribution requirement:
- each `IntentAttempt` carries `cost_attribution_id` for end-to-end debit matching.

6. Attempt ID determinism:
- `attempt_id` must be derived via the fixed canonical formula above.
