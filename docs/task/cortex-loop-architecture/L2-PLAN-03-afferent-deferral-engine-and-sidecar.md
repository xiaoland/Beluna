# L2 Plan 03 - Afferent Deferral Engine and Sidecar
- Task: `cortex-loop-architecture`
- Micro-task: `03-afferent-deferral-engine-and-sidecar`
- Stage: `L2`
- Date: `2026-03-01`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Goal and Scope
Goal:
1. Introduce a deterministic deferral engine inside Afferent Pathway.
2. Keep sidecar strictly observe-only.
3. Provide a pathway-owned rule-control port that Cortex Primary will wrap as tool operations.
4. Relocate afferent pathway under Stem module ownership.

In scope:
1. `stem::afferent_pathway` module design and ports.
2. Rule model and single-rule overwrite/reset semantics.
3. Deferred FIFO and overflow-eviction behavior.
4. Sidecar event contract.

Out of scope:
1. Primary tool schema itself (implemented in micro-task `04`).
2. Prompt/tooling UX docs refresh (micro-task `08`).

## 2) Module Topology and Ownership
1. Afferent pathway becomes Stem submodule:
- from: `core/src/afferent_pathway.rs`
- to: `core/src/stem/afferent_pathway.rs`
2. Public imports move to `crate::stem::afferent_pathway::*`.
3. Ownership split:
- Stem module owns implementation and lifecycle wiring.
- Cortex owns consumer handle and rule-control usage via injected trait object.

## 3) Port Contracts
### 3.1 Producer/Consumer Path
```rust
pub trait SenseIngressPort {
    async fn send(&self, sense: Sense) -> Result<(), AfferentPathwayError>;
}

pub struct SenseConsumerHandle {
    pub async fn recv(&mut self) -> Option<Sense>;
    pub fn try_recv(&mut self) -> Result<Sense, TryRecvError>;
}
```

### 3.2 Rule-Control Port (Pathway-Owned)
```rust
#[async_trait]
pub trait AfferentRuleControlPort: Send + Sync {
    async fn overwrite_rule(&self, input: DeferralRuleOverwriteInput) -> Result<RuleRevision, RuleControlError>;
    async fn reset_rules(&self) -> RuleRevision;
    async fn snapshot_rules(&self) -> DeferralRuleSetSnapshot;
}
```

Semantics:
1. `overwrite_rule` overwrites one rule by `rule_id` (upsert).
2. `reset_rules` clears all rules.
3. No rule removal API.

### 3.3 Sidecar Port (Observe-Only)
```rust
pub trait AfferentSidecarPort {
    fn subscribe(&self) -> AfferentSidecarSubscription;
}
```

Constraints:
1. Subscription is read-only.
2. Sidecar backpressure must never block sense delivery or control operations.

## 4) Data Structures
```rust
pub struct DeferralRule {
    pub rule_id: String,
    pub min_weight: Option<f64>,
    pub fq_sense_id_pattern: Option<String>,
    // compiled regex stored internally
}

pub struct DeferredSenseEntry {
    pub sense: Sense,
    pub deferred_at_ms: u64,
}

pub struct DeferralState {
    pub revision: u64,
    pub rules_by_id: BTreeMap<String, DeferralRuleRuntime>,
    pub deferred_fifo: VecDeque<DeferredSenseEntry>,
}
```

Rule validation:
1. At least one selector required (`min_weight` or regex).
2. `min_weight` must be within `[0,1]`.
3. Regex must compile.

## 5) Matching and Scheduling Algorithm
Decision function (`should_defer`):
1. Build `fq_sense_id = "{endpoint_id}/{neural_signal_descriptor_id}"`.
2. A sense is deferred if any active rule matches:
- `sense.weight < min_weight` when `min_weight` is set.
- regex matches `fq_sense_id` when pattern is set.
- both selectors must pass when both are present.

Ingress scheduling:
1. On new sense:
- if matched: push into `deferred_fifo`.
- else: forward to consumer queue immediately.
2. On overflow (`len > max_deferring_nums`):
- evict oldest deferred entries until within cap.
- emit warning log + sidecar eviction events.

Release scheduling:
1. Triggered after each `overwrite_rule` and `reset_rules`.
2. FIFO-strict release:
- repeatedly inspect front entry.
- if front still matches any active rule: stop.
- else pop front and forward.
3. This preserves deferred buffer order deterministically.

## 6) Concurrency Model
1. Internal pathway scheduler task owns mutable `DeferralState`.
2. Inputs multiplexed via `tokio::select!`:
- ingress sense queue
- control command queue (overwrite/reset/snapshot)
- shutdown signal.
3. Control commands are serialized with ingress events by single-owner loop.
4. Sidecar event emission uses non-blocking channel strategy (`broadcast` or best-effort fanout).

## 7) Sidecar Event Contract
```rust
pub enum AfferentSidecarEvent {
    RuleOverwritten { revision: u64, rule_id: String },
    RulesReset { revision: u64, removed_count: usize },
    SenseDeferred { sense_instance_id: String, rule_ids: Vec<String>, deferred_len: usize },
    SenseReleased { sense_instance_id: String, deferred_len: usize },
    SenseEvicted { sense_instance_id: String, reason: String, deferred_len: usize },
}
```

## 8) File and Interface Impact
1. Create/move: `core/src/stem/afferent_pathway.rs`.
2. Update exports: `core/src/stem.rs`.
3. Update imports in:
- `core/src/main.rs`
- `core/src/spine/runtime.rs`
- `core/src/continuity/engine.rs`
- any module currently importing `crate::afferent_pathway`.
4. Add rule-control trait injection point for Cortex runtime/Primary wiring (actual tool wrapping in micro-task `04`).

## 9) Risks and Mitigations
1. Regex-heavy rules may impact hot path.
- Mitigation: compile once at overwrite time; evaluate precompiled regex only.
2. Burst release after reset can flood Cortex.
- Mitigation: egress queue backpressure is respected; release loop yields on send await.
3. Sidecar volume overhead.
- Mitigation: best-effort non-blocking publish and bounded channel.

## 10) L2 Exit Criteria
1. Afferent is modeled as `stem::afferent_pathway` submodule.
2. Pathway-owned rule-control port contract is frozen.
3. `overwrite_rule` semantics are explicitly single-rule upsert.
4. Deferred FIFO + overflow eviction semantics are deterministic.
5. Sidecar remains observe-only by type and API boundaries.

Status: `READY_FOR_REVIEW`
