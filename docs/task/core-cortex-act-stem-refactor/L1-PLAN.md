# L1 Plan - Core Cortex Act Stem Refactor (High-Level Strategy)
- Task Name: `core-cortex-act-stem-refactor`
- Stage: `L1` (high-level strategy)
- Date: `2026-02-13`
- Status: `DRAFT_FOR_APPROVAL`

## 0) Inputs Locked From L0 Approval
User-confirmed constraints (authoritative):
1. Admission semantics are removed completely.
2. `IntentAttempt` is renamed/replaced by simplified `Act`.
3. `Act` is not enriched with extra pipeline-owned fields.
4. Sense queue uses bounded Rust MPSC native behavior (blocking senders when full).
5. Pipeline control contract is only `Continue | Break`.
6. `Break` stops dispatch for current `Act` only.
7. Cortex is pure in ownership semantics:
   - no internal persistence,
   - no side-effect to other components,
   - AI Gateway usage is allowed.
8. `CognitionState` (goal stack) is persisted in Continuity and managed by Cortex output.
9. Add/keep canonical control senses:
   - `sleep` (shutdown trigger),
   - `new_capabilities` (incremental patch),
   - `drop_capabilities` (incremental removal patch).
10. Hard delete Admission now; breaking changes are allowed.
11. No separate Act queue; Stem dispatches Acts inline in serial order.
12. On shutdown, `main` blocks until `sleep` sense is enqueued.
13. On shutdown, ingress is gated first to forbid new senses.
14. Capability patch conflict policy is arrival-order-wins.

## 1) Strategy Summary
Refactor runtime from:
`Sense -> Cortex -> IntentAttempt -> Admission -> AdmittedAction -> Spine`
to:
`Sense -> Cortex -> Act -> Dispatch Pipeline(Ledger -> Continuity -> Spine)`.

Core direction:
1. Keep Cortex stateless and side-effect free through explicit input/output contracts.
2. Make Stem the orchestrator of all cross-component effects.
3. Keep Continuity as owner of persisted cognition state (goal stack).
4. Keep Spine as owner of body endpoint capabilities; Stem composes full physical state each cycle.
5. Replace current unbounded ingress + internal deque behavior with bounded MPSC producer backpressure semantics.

## 2) Target Architecture

```text
Producers: BodyEndpoint / Spine / Continuity / Ledger
  -> SenseQueue (bounded MPSC, blocking senders)
  -> Stem (consumer)
     1) on Sense:
        - intercept control senses
        - compose PhysicalState (ledger + merged capabilities)
        - load CognitionState from Continuity
        - call cortex(sense, physical_state, cognition_state)
        - persist new CognitionState via Continuity
        - directly dispatch returned Acts through Ledger -> Continuity -> Spine
        - each stage returns Continue or Break (current Act only)
```

Main process topology:
1. `main` initializes Ledger, Continuity, Spine, bounded Sense queue, and Stem loop.
2. `main` listens for `SIGINT`/`SIGTERM`; on signal gates ingress and then emits `sleep` sense.
3. `sleep` sense is consumed by Stem and stops Stem loop; cleanup/flush follows.

## 3) Boundary Ownership
1. Cortex owns:
   - deterministic state transition over `(sense, physical_state, cognition_state)`,
   - emitted `Act[]`,
   - returned `new_cognition_state`.
2. Stem owns:
   - event orchestration,
   - sense interception,
   - physical state assembly,
   - act dispatch sequencing,
   - cleanup coordination.
3. Continuity owns:
   - persistence of `CognitionState` (goal stack),
   - non-semantic operational state,
   - capability contributions to physical-state composition.
4. Ledger owns:
   - survival/accounting state contribution to physical state,
   - dispatch-stage checks/side effects according to pipeline contract.
5. Spine owns:
   - body endpoint routing and body-managed capability catalog.

## 4) High-Level Contract Direction
1. `Sense` becomes explicit runtime envelope with control variants:
   - domain senses,
   - `sleep`,
   - `new_capabilities` patch,
   - `drop_capabilities` patch.
2. `PhysicalState` is explicit and cycle-scoped:
   - ledger status snapshot,
   - merged capability catalog snapshot.
3. `CognitionState` is explicit and persistence-backed by Continuity.
4. `Act` is Cortex output and pipeline input; no admission-derived expansion.
5. Dispatch stage result is binary:
   - `Continue` to next stage,
   - `Break` for current `Act`.

## 5) Capability Composition Strategy
1. Spine remains authoritative for body endpoint capability lifecycle.
2. Continuity and Ledger expose their own capability contributions.
3. Stem builds merged capability snapshot at sense-processing time.
4. Capability patch senses update capability state immediately before Cortex call in the same cycle.

High-level merge rule:
1. apply patches in arrival order,
2. compute merged snapshot,
3. pass merged snapshot to Cortex as part of `PhysicalState`.

## 6) Afferent Queue & Backpressure Strategy
1. Use bounded Tokio MPSC for Sense queue.
2. Producers block on full queue (native behavior, no drop-oldest policy in this refactor).
3. Stem is the single consumer.
4. Shutdown path keeps queue semantics explicit:
   - first gate ingress to forbid new senses,
   - block until `sleep` is enqueued,
   - Stem exits on consuming `sleep`,
   - then perform component cleanup.

## 7) Dispatch Pipeline Strategy
Pipeline order is fixed:
1. Ledger stage.
2. Continuity stage.
3. Spine stage.

Pipeline semantics:
1. Stage returns `Continue` or `Break`.
2. `Break` aborts only current `Act` traversal.
3. Next `Act` continues independently.
4. Pipeline must preserve deterministic order of `Act` processing.
5. Dispatch happens inline in Stem loop (no separate `ActQueue`).

## 8) Admission Removal Strategy
1. Remove `core/src/admission/*` module and exports.
2. Replace all admission-dependent flows in:
   - `brainstem`,
   - `continuity`,
   - tests and docs.
3. Replace terms in contracts/docs:
   - `IntentAttempt` -> `Act`,
   - remove `AdmittedAction`/`AdmissionReport` boundary references where obsolete.
4. Update Spine boundary contracts from admitted-only semantics to pipeline-consumable act execution semantics.

## 9) Risk Focus & Mitigations
1. Risk: removing Admission breaks settlement invariants unintentionally.
   - Mitigation: preserve ledger invariants explicitly in dispatch-stage contracts/tests.
2. Risk: bounded blocking queue can stall producers under heavy load.
   - Mitigation: define capacity sizing and observability at runtime config + tests.
3. Risk: capability patch ordering drift.
   - Mitigation: deterministic patch application order in Stem.
4. Risk: broad rename/removal causes inconsistent public API/docs.
   - Mitigation: ruthless one-pass contract/type/doc sweep in implementation phase.

## 10) Deliverables Expected From L2
L2 should define:
1. precise interface signatures:
   - `cortex(...) -> (acts, new_cognition_state)`,
   - queue and stem interfaces,
   - stage dispatch interfaces.
2. canonical data structures:
   - `Sense` variants/payloads,
   - `PhysicalState`,
   - `CognitionState` goal stack schema,
   - `Act`,
   - `DispatchDecision`.
3. algorithms:
   - Stem event loop logic over `sense_rx`,
   - capability patch application/merge,
   - per-act pipeline dispatch and break behavior.
4. module/file map for admission deletion and replacement wiring.
5. test strategy for:
   - bounded queue behavior,
   - control-sense interception,
   - pipeline break semantics,
   - cognition persistence path.

## 11) L1 Decision Status
All L1 control decisions are resolved by user:
1. `sleep` enqueue on shutdown is blocking.
2. ingress is gated before `sleep` enqueue.
3. capability patch conflict policy is arrival-order-wins.

## 12) L1 Exit Criteria
L1 is complete when accepted:
1. new runtime topology and ownership split are locked,
2. bounded MPSC backpressure model is locked,
3. capability patch senses and merge direction are locked,
4. `Continue | Break` act-dispatch semantics are locked,
5. Admission hard-deletion path is accepted,
6. shutdown enqueue/gating policy is locked,
7. capability patch conflict policy is locked.

Status: `READY_FOR_L2_APPROVAL`
