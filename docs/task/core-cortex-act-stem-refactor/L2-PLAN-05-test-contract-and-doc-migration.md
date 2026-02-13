# L2 Plan 05 - Test Contract And Documentation Migration
- Task Name: `core-cortex-act-stem-refactor`
- Stage: `L2`
- Focus: contract-to-test matrix and docs migration plan
- Status: `DRAFT_FOR_APPROVAL`

## 1) Test Migration Strategy
Use ruthless replacement:
1. delete admission test suite.
2. add stem-focused integration tests.
3. update cortex/continuity/spine/ledger tests to new contracts.

## 2) Test Delta Map

### 2.1 Delete
1. `core/tests/admission_bdt.rs`
2. `core/tests/admission/mod.rs`
3. `core/tests/admission/admission.rs`

### 2.2 Add
1. `core/tests/stem_bdt.rs`
2. `core/tests/stem/mod.rs`
3. `core/tests/stem/loop.rs`
4. `core/tests/stem/shutdown.rs`
5. `core/tests/stem/capability_patch.rs`
6. `core/tests/stem/dispatch_pipeline.rs`

### 2.3 Update
1. `core/tests/cortex/reactor.rs`
2. `core/tests/cortex/clamp.rs`
3. `core/tests/continuity/debits.rs`
4. `core/tests/spine/contracts.rs`
5. `core/tests/cortex_continuity_flow.rs`
6. `core/tests/ledger/ledger.rs`

## 3) Contract-To-Test Matrix

1. Cortex stateless boundary:
- contract: no persistence, no component side effects.
- tests:
  - same `(sense, physical_state, cognition_state)` yields deterministic output under mocked ports.
  - `sleep` is never passed into cortex by stem.

2. Sense queue backpressure:
- contract: bounded MPSC blocks senders when full.
- tests:
  - sender blocks until receiver drains.
  - no drop-oldest behavior.

3. Shutdown flow:
- contract: gate ingress first, then blocking sleep enqueue.
- tests:
  - post-gate producer sends are rejected.
  - `sleep` still enqueues and terminates stem.

4. Capability patch/drop:
- contract: arrival-order-wins; drop tombstones route.
- tests:
  - new patch applied before same-cycle cortex call.
  - drop removes capability from composed physical state.
  - late new patch reactivates dropped capability.

5. Serial dispatch pipeline:
- contract: order is Ledger -> Continuity -> Spine, `Break` current Act only.
- tests:
  - Ledger break skips Continuity/Spine for that act.
  - Continuity break skips Spine for that act.
  - following act still dispatches.
  - dispatch order preserves act index sequence.

6. Settlement invariants:
- contract: ledger terminality/idempotency preserved after spine outcomes.
- tests:
  - applied/rejected events map to settle/refund deterministically.
  - duplicate reference settlement remains idempotent.

## 4) Documentation Migration

### 4.1 Update Core Narrative Docs
1. `docs/overview.md`
- replace old flow with `Sense -> Cortex -> Act -> serial pipeline`.

2. `docs/glossary.md`
- remove `IntentAttempt`, `AdmittedAction`, `AdmissionReport` terms.
- add:
  - `Act`,
  - `Sense::Sleep`,
  - `Sense::NewCapabilities`,
  - `Sense::DropCapabilities`,
  - `Stem`.

### 4.2 Update Feature/Module/Contract Indexes
1. `docs/features/README.md`
- remove Admission feature index entry.
2. `docs/modules/README.md`
- remove Admission module index entry.
3. `docs/contracts/README.md`
- remove Admission contract index entry.

### 4.3 Update/Replace Specific Packages
1. remove or archive:
   - `docs/features/admission/*`
   - `docs/modules/admission/*`
   - `docs/contracts/admission/*`
2. update:
   - `docs/features/cortex/*`
   - `docs/features/continuity/*`
   - `docs/features/spine/*`
   - `docs/features/ledger/*`
3. add/extend Stem contracts under:
   - `docs/contracts/continuity/*` or a dedicated stem contract package.

## 5) Validation Commands (L3 execution guidance)
Primary targeted runs:
1. `cd core && cargo test stem:: -- --nocapture`
2. `cd core && cargo test cortex:: -- --nocapture`
3. `cd core && cargo test continuity:: -- --nocapture`
4. `cd core && cargo test spine:: -- --nocapture`
5. `cd core && cargo test ledger:: -- --nocapture`

Full pass after targeted:
1. `cd core && cargo test -- --nocapture`

## 6) L2-05 Exit Conditions
1. every removed boundary has test coverage replacement,
2. doc index and terminology migration is explicit,
3. L3 has concrete validation commands and success criteria.

Status: `READY_FOR_REVIEW`
