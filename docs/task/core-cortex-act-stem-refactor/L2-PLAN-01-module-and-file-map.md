# L2 Plan 01 - Module And File Map
- Task Name: `core-cortex-act-stem-refactor`
- Stage: `L2`
- Focus: Source/Test/Doc file-level migration map and dependency boundaries
- Status: `DRAFT_FOR_APPROVAL`

## 1) Source File Delta Map

### 1.1 Delete
1. `core/src/admission/AGENTS.md`
2. `core/src/admission/affordance.rs`
3. `core/src/admission/mod.rs`
4. `core/src/admission/resolver.rs`
5. `core/src/admission/types.rs`

### 1.2 Add
1. `core/src/stem.rs`
- canonical Stem loop and serial Act dispatch pipeline.

2. `core/src/runtime_types.rs`
- shared runtime contracts:
  - `Sense`,
  - `PhysicalState`,
  - `CognitionState`,
  - `Act`,
  - capability patch/drop payloads,
  - `DispatchDecision`.

3. `core/src/ingress.rs`
- ingress-gated bounded-sender wrapper used by all producers and shutdown flow.

### 1.3 Update (Major)
1. `core/src/main.rs`
- replace `brainstem::run` entry with `stem::run`.
- add signal listener flow: gate ingress then blocking `sleep` enqueue.

2. `core/src/lib.rs`
- remove `pub mod admission`.
- add `pub mod runtime_types`, `pub mod ingress`, `pub mod stem`.

3. `core/src/brainstem.rs`
- remove (or keep as thin compatibility shim forwarding to `stem` during same PR; final state should not own loop logic).

4. `core/src/cortex/types.rs`
- remove `ReactionInput`/`ReactionResult` and `IntentAttempt` dependencies.
- adopt shared runtime contracts from `runtime_types`.

5. `core/src/cortex/ports.rs`
- expose `cortex(sense, physical_state, cognition_state)` boundary API.

6. `core/src/cortex/pipeline.rs`
- adapt internals to produce `Vec<Act>` and `new_cognition_state`.

7. `core/src/cortex/clamp.rs`
- output `Act` (not `IntentAttempt`).
- remove admission-type imports.

8. `core/src/continuity/state.rs`
- remove internal `sense_queue` and `neural_signal_queue` ownership.
- add persisted `cognition_state`.
- add capability overlay+tombstone state for patch and drop senses.

9. `core/src/continuity/types.rs`
- remove `AdmissionReport`, `NeuralSignalBatch`, admission-attribution wrappers.
- add continuity dispatch and cognition persistence records.

10. `core/src/continuity/engine.rs`
- remove admission orchestration and `process_attempts`.
- expose:
  - cognition snapshot/persist API,
  - capability patch/drop apply API,
  - continuity dispatch-stage API,
  - continuity capability contribution snapshot.

11. `core/src/spine/types.rs`
- replace admitted-action contracts with `Act`-dispatch request contracts.
- retain ordered event and settlement linkage invariants (`reserve_entry_id`, `cost_attribution_id`).

12. `core/src/spine/ports.rs`
- rename `execute_admitted` to single-act dispatch API (serial call semantics).

13. `core/src/spine/router.rs`
- dispatch Act request and emit ordered per-act events.

14. `core/src/spine/noop.rs`
- align with new Act execution request API.

15. `core/src/spine/adapters/wire.rs`
- remove admission-feedback wire envelope.
- support patch-based capability control senses.

16. `core/src/spine/adapters/unix_socket.rs`
- emit/control `Sense` variants through ingress wrapper.
- honor ingress gate during shutdown.

17. `core/src/body/mod.rs`
- switch endpoint invocation payload from old admitted action type to new Act execution request type.

18. `core/src/config.rs`
- loop config simplified:
  - keep `sense_queue_capacity`,
  - remove `neural_signal_queue_capacity`,
  - remove batch-window and batch-size controls tied to old batching path.

19. `core/beluna.schema.json`
- remove old loop batch fields.
- retain `loop.sense_queue_capacity`.

## 2) Test File Delta Map

### 2.1 Delete
1. `core/tests/admission/mod.rs`
2. `core/tests/admission/admission.rs`
3. `core/tests/admission_bdt.rs`

### 2.2 Add
1. `core/tests/stem/mod.rs`
2. `core/tests/stem/loop.rs`
3. `core/tests/stem/shutdown.rs`
4. `core/tests/stem/capability_patch.rs`
5. `core/tests/stem/dispatch_pipeline.rs`
6. `core/tests/stem_bdt.rs`

### 2.3 Update
1. `core/tests/cortex/*`
- assert new pure-boundary contract (`Sense + PhysicalState + CognitionState -> Act[] + new CognitionState`).

2. `core/tests/continuity/*`
- remove admission expectations.
- add cognition persistence and patch/drop behavior assertions.

3. `core/tests/spine/*`
- move from admitted-batch tests to single-act dispatch tests.

4. `core/tests/cortex_continuity_flow.rs`
- replace old admission bridge with inline serial dispatch through Stem.

## 3) Doc File Delta Map
1. Update:
   - `docs/overview.md`
   - `docs/glossary.md`
   - `docs/features/README.md`
   - `docs/modules/README.md`
   - `docs/contracts/README.md`
2. Remove Admission doc package references:
   - `docs/features/admission/*`
   - `docs/modules/admission/*`
   - `docs/contracts/admission/*`
3. Add or update contracts for:
   - Stem loop,
   - control senses (`sleep`, `new_capabilities`, `drop_capabilities`),
   - serial pipeline dispatch behavior.

## 4) Dependency Direction Rules
1. `runtime_types` has no dependency on Admission module.
2. `cortex` depends on runtime contracts and AI ports only.
3. `stem` orchestrates cross-module effects and is the only component that ties cortex/ledger/continuity/spine together.
4. `continuity`, `ledger`, and `spine` do not depend on Cortex internals.
5. `spine` remains transport-ignorant in core modules; adapter-specific logic stays under `core/src/spine/adapters/*`.

## 5) L2-01 Exit Conditions
1. file-level migration is explicit enough to execute in L3,
2. delete/add/update boundaries are unambiguous,
3. dependency direction forbids accidental re-introduction of admission semantics.

Status: `READY_FOR_REVIEW`
