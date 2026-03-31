# L2-02 - Domain Model And Contracts
- Task Name: `cortex-mvp`
- Stage: `L2` detailed file
- Date: `2026-02-11`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Canonical Reactor Types
Rust-level direction for reactor contracts.

```rust
pub type ReactionId = u64;
pub type SenseId = String;
pub type AttemptId = String;
pub type AttentionTag = String;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReactionInput {
    pub reaction_id: ReactionId,
    pub sense_window: Vec<SenseDelta>,                  // ordered, bounded
    pub env_snapshots: Vec<EndpointSnapshot>,           // latest snapshot blobs
    pub admission_feedback: Vec<AdmissionOutcomeSignal>,// non-semantic codes + correlation
    pub capability_catalog: CapabilityCatalog,
    pub limits: ReactionLimits,
    pub context: IntentContext,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReactionResult {
    pub reaction_id: ReactionId,
    pub based_on: Vec<SenseId>,
    pub attention_tags: Vec<AttentionTag>,
    pub attempts: Vec<IntentAttempt>, // empty => noop fallback
}
```

## 2) Input Model Details
### Sense delta window
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SenseDelta {
    pub sense_id: SenseId,      // stable id from upstream continuity
    pub source: String,
    pub payload: serde_json::Value,
}
```

Invariants:
1. `sense_window` must be pre-ordered by upstream.
2. `sense_id` must be unique within one `ReactionInput`.
3. size must be <= `limits.max_sense_items`.

### Environment snapshots
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EndpointSnapshot {
    pub endpoint_key: String,
    pub blob: serde_json::Value, // opaque allowed
    pub truncated: bool,         // deterministic truncation marker
    pub blob_bytes: usize,
}
```

Invariants:
1. `blob_bytes` <= `limits.max_snapshot_bytes_per_item`.
2. one entry per `endpoint_key` per cycle.
3. truncation must be deterministic and stable for same input blob.

### Admission feedback correlation
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdmissionOutcomeSignal {
    pub attempt_id: AttemptId,  // required correlation key
    pub code: String,           // non-semantic outcome code
}
```

Invariants:
1. semantic intent labels are forbidden in `code`.
2. feedback path must always include `attempt_id`.

## 3) Intent Context (Distributed Ownership)
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntentContext {
    pub constitutional: Vec<ConstitutionalIntent>,
    pub environmental: Vec<EnvironmentalIntentSignal>,
    pub emergent_candidates: Vec<EmergentIntentCandidate>,
}
```

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstitutionalIntent {
    pub intent_key: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnvironmentalIntentSignal {
    pub signal_key: String,
    pub constraint_code: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmergentIntentCandidate {
    pub candidate_key: String,
    pub summary: String,
    pub provenance: String,
}
```

Rule:
- Cortex may arbitrate among these inputs but does not persist them durably.

## 4) Capability Catalog Model
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityCatalog {
    pub version: String,
    pub affordances: Vec<AffordanceCapability>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AffordanceCapability {
    pub affordance_key: String,
    pub allowed_capability_handles: Vec<String>,
    pub payload_schema: serde_json::Value,
    pub max_payload_bytes: usize,
    pub default_resources: RequestedResources,
}
```

Routing contract:
1. route choice is done in Cortex cognition.
2. clamp drops unknown `affordance_key`.
3. clamp drops unsupported `capability_handle`.

## 5) Reaction Limits And Budgets
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReactionLimits {
    pub max_sense_items: usize,
    pub max_snapshot_items: usize,
    pub max_snapshot_bytes_per_item: usize,
    pub max_attempts: usize,
    pub max_payload_bytes: usize,
    pub max_cycle_time_ms: u64,
    pub max_primary_calls: u8,   // fixed to 1
    pub max_sub_calls: u8,       // small, default 2
    pub max_repair_attempts: u8, // fixed to 1
    pub max_primary_output_tokens: u64,
    pub max_sub_output_tokens: u64,
}
```

Hard bounds:
1. `max_primary_calls` must equal 1 at runtime validation.
2. `max_repair_attempts` must be <= 1.
3. clamp truncates attempts by deterministic priority and never exceeds `max_attempts`.

## 6) Prose IR + Compile Draft Types
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProseIr {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttemptDraft {
    pub intent_span: String,             // must be traceable to IR
    pub based_on: Vec<SenseId>,
    pub attention_tags: Vec<AttentionTag>,
    pub affordance_key: String,
    pub capability_handle: String,
    pub payload_draft: serde_json::Value,
    pub requested_resources: RequestedResources,
}
```

No-intent-drift rule:
1. each draft must include `intent_span`.
2. clamp may reject draft if `intent_span` is empty or non-traceable under configured parser checks.

## 7) `IntentAttempt` Contract Update
Direction for `core/src/admission/types.rs`:

```rust
pub struct IntentAttempt {
    pub attempt_id: AttemptId,
    pub cycle_id: u64,
    pub commitment_id: String,
    pub goal_id: String,
    pub planner_slot: u16,
    pub based_on: Vec<SenseId>,        // NEW: required world-relative grounding
    pub affordance_key: String,
    pub capability_handle: String,
    pub normalized_payload: serde_json::Value,
    pub requested_resources: RequestedResources,
    pub cost_attribution_id: String,
}
```

Required invariants:
1. `attempt_id` is mandatory and deterministic.
2. `based_on` is mandatory and non-empty for non-noop outputs.
3. every `based_on` id must exist in `ReactionInput.sense_window`.
4. feedback must preserve `attempt_id` for correlation.

## 8) Deterministic ID Derivation
Keep deterministic derivation with updated canonical fields:
1. `cost_attribution_id = hash(reaction_id, affordance_key, capability_handle, based_on, planner_slot)`
2. `attempt_id = hash(reaction_id, based_on, affordance_key, capability_handle, normalized_payload, requested_resources, cost_attribution_id)`

Requirements:
1. canonical JSON key ordering before hashing.
2. no wall-clock/random data in hash payload.

## 9) Business Output Purity Rule
`ReactionResult` must not include operational telemetry.

Telemetry path direction:
1. emit metrics/events through separate observer port.
2. do not add debug counters to attempt business payload.

## 10) L2-02 Exit Criteria
This file is complete when:
1. all reactor business contracts are typed and bounded,
2. `attempt_id` + `based_on` + feedback-correlation requirements are explicit,
3. distributed intent context and statelessness boundaries are encoded.
