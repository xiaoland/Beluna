# L0 Plan - Core Cortex Act Stem Refactor
- Task Name: `core-cortex-act-stem-refactor`
- Stage: `L0` (request + context analysis only)
- Date: `2026-02-13`
- Status: `REVISED_WITH_USER_DECISIONS`

## 1) Problem Deconstruction
The request is a runtime architecture rewrite, not a local refactor.

Target outcomes:
1. Remove Admission boundary and remove admitted-intent stage.
2. Rename `IntentAttempt` semantic role to `Act`.
3. Make Cortex callable as:
   - `fn cortex(sense, physical_state, cognition_state) -> (acts, new_cognition_state)`
4. Introduce explicit `physical_state`:
   - ledger status + capabilities.
   - capabilities are composed from Spine, Continuity, Ledger, and Body Endpoints (Body Endpoint capabilities managed by Spine).
5. Introduce explicit `cognition_state`:
   - goal stack now, extendable later.
   - persisted in Continuity and managed by Cortex via `new_cognition_state` output.
6. Use Rust MPSC as bounded Sense queue with backpressure:
   - producers: `[BodyEndpoint, Spine, Continuity, Ledger]`
   - consumer: Cortex via Stem loop.
   - queue pressure behavior uses native bounded MPSC semantics (senders block when full).
7. Rebuild runtime control flow:
   - `main` starts Continuity/Ledger/Spine and sense queue.
   - `main` starts Stem loop.
   - `main` handles OS signal and injects `sleep` sense.
8. Stem behavior:
   - wait next sense.
   - intercept `sleep` and break without calling Cortex.
   - intercept `new_capabilities`, apply incremental capability patch, then call Cortex with updated physical state.
   - intercept `drop_capabilities`, remove capabilities by patch payload, then call Cortex with updated physical state.
   - dispatch returned acts through pipeline `(Ledger -> Continuity -> Spine)` where each stage may continue or break.

## 2) Context Collection (Sub-agent Style)
To reduce cognitive load, analysis was split into parallel tracks.

1. Track A (local architecture scan):
   - runtime wiring: `core/src/main.rs`, `core/src/brainstem.rs`, `core/src/config.rs`
   - module boundaries: `core/src/cortex/*`, `core/src/continuity/*`, `core/src/admission/*`, `core/src/spine/*`, `core/src/ledger/*`
   - docs/contracts/features for Cortex/Continuity/Admission/Spine/Ledger
   - test surfaces in `core/tests/*`

2. Track B (external references via Firecrawl):
   - Tokio bounded MPSC semantics/backpressure/clean shutdown:
     - https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html
   - Tokio Unix signal listener behavior and caveats:
     - https://docs.rs/tokio/latest/tokio/signal/unix/struct.Signal.html
     - https://docs.rs/tokio/latest/tokio/signal/unix/fn.signal.html

## 3) Current Codebase Reality

### 3.1 Main/runtime structure today
1. `main` loads config and delegates to `brainstem::run`.
2. `brainstem` owns event loop, batching, and signal handling.
3. SIGINT/SIGTERM currently break loop directly, not via injected `sleep` sense.

### 3.2 Sense ingress and backpressure today
1. Ingress channels are `tokio::sync::mpsc::unbounded_channel` (no transport-level backpressure).
2. Backpressure is currently simulated by bounded `VecDeque` in Continuity state (`sense_queue_capacity`) with drop-oldest behavior.
3. This differs from requested bounded MPSC producer backpressure semantics (blocking senders on full queue).

### 3.3 Cortex shape today
1. Cortex consumes `ReactionInput` (sense window + env snapshots + admission feedback + capability catalog + context).
2. Cortex pipeline is async and AI-backed (`primary -> extractor -> clamp -> optional filler`).
3. Output is `ReactionResult { attempts: Vec<IntentAttempt> }`.

### 3.4 Admission dependency today
1. Admission is a dedicated mechanical gate:
   - `IntentAttempt[] -> AdmissionReport + AdmittedActionBatch`.
2. Continuity orchestrates admission, spine execution, settlement reconciliation, and debit ingestion.
3. Spine contract explicitly accepts admitted actions only.

### 3.5 Spine/capability handling today
1. Spine owns runtime capability catalog snapshot.
2. Brainstem bridges Spine catalog to Cortex capability catalog.
3. Body endpoint registration/unregistration updates routing + capability view.

### 3.6 Cognition state today
1. No explicit persisted `cognition_state` goal stack in runtime loop.
2. Current Cortex is stateless per reaction input; long-lived operational state sits in Continuity/Ledger.

## 4) Fit-Gap Matrix Against Requested Target
1. Remove Admission: `GAP`
   - Admission module is deeply wired through Continuity, Brainstem, tests, and docs.
2. Rename `IntentAttempt` to `Act`: `GAP`
   - type is widely referenced across Cortex/Admission/Continuity/tests/docs.
3. Pure Cortex function signature: `PARTIAL`
   - current interface is function-like (`react_once(input)`), but input/output schema differs and includes admission feedback; implementation is async AI-calling pipeline.
4. Physical state explicit composition (ledger + capabilities): `PARTIAL`
   - pieces exist separately, but no unified `PhysicalState` struct.
5. Cognition state explicit goal stack: `GAP`
   - no dedicated state container in loop.
6. Bounded MPSC afferent pathway with producer backpressure: `GAP`
   - current ingress is unbounded channel + internal bounded deque.
7. Stem special sense interception (`sleep`, `new_capabilities`, `drop_capabilities`): `PARTIAL`
   - capabilities updates are handled indirectly via endpoint messages today; no canonical `sleep`/`drop_capabilities` sense path.

## 5) Migration Blast Radius
Primary impact surfaces:
1. Runtime loop and wiring:
   - `core/src/brainstem.rs`
   - `core/src/main.rs`
   - `core/src/config.rs`
2. Core domain modules:
   - `core/src/cortex/*`
   - `core/src/continuity/*`
   - `core/src/spine/*`
   - `core/src/ledger/*`
   - removal or deprecation path for `core/src/admission/*`
3. Public exports:
   - `core/src/lib.rs`
4. Tests:
   - `core/tests/admission/*`
   - `core/tests/cortex/*`
   - `core/tests/continuity/*`
   - `core/tests/cortex_continuity_flow.rs`
   - `core/tests/spine/*` (contract updates)
5. Docs/contracts/features/glossary:
   - references to Admission, `IntentAttempt`, `AdmittedAction`, and current flow.

## 6) Architectural Trade-offs Identified
Locked decisions from user:
1. Admission is removed entirely (hard delete, breaking changes allowed).
2. `Act` is simplified `IntentAttempt` replacement; no extra pipeline-enrichment fields required.
3. Pipeline decision contract is only `Continue` and `Break`.
4. Sense queue backpressure uses bounded MPSC native behavior (blocking senders on full queue).
5. Pure Cortex means stateless/no persistence and no side-effects to other components; AI Gateway usage is allowed.
6. Goal stack (`cognition_state`) is persisted in Continuity and managed by Cortex outputs.
7. Pipeline `Break` stops current `Act` dispatch only.
8. Capability update senses:
   - `new_capabilities` carries incremental patch payload.
   - `drop_capabilities` is added for capability removal patch payload.

Remaining trade-offs for implementation design:
1. Capability composition source-of-truth:
   - option A: Spine owns full capability catalog, Continuity/Ledger contribute overlays.
   - option B: Stem computes merged snapshot each cycle.
   - trade-off: ownership clarity vs flexibility.

## 7) External Findings Relevant to Design
1. Tokio bounded MPSC is many-to-one and provides backpressure by capacity-bound send behavior.
   - Source: Tokio docs (`tokio::sync::mpsc`), bounded channel section.
2. Tokio MPSC recommends explicit clean shutdown (`Receiver::close` then drain).
   - Source: Tokio docs (`tokio::sync::mpsc`), clean shutdown section.
3. Tokio Unix signal listeners are cancel-safe in `select!` via `Signal::recv`.
   - Source: `tokio::signal::unix::Signal::recv`.
4. Tokio signal caveat: first listener overrides default signal behavior for process lifetime.
   - Source: `tokio::signal::unix::Signal` caveats.

These findings support:
1. bounded sense queue + explicit shutdown/drain path,
2. signal-to-sense translation design with controlled termination semantics.

## 8) L0 Recommendation (Scope Boundary for L1)
L1 should design a **single new canonical loop model** with these boundaries:
1. Introduce explicit runtime data models:
   - `Sense`
   - `PhysicalState`
   - `CognitionState`
   - `Act`
2. Replace `IntentAttempt -> Admission -> AdmittedAction` flow with:
   - `Cortex -> Act[] -> stage pipeline (Ledger, Continuity, Spine)`.
3. Move ingress from unbounded channel + internal queue to bounded MPSC sense queue with explicit producer policy.
4. Add canonical special senses:
   - `sleep`
   - `new_capabilities` (incremental patch)
   - `drop_capabilities` (incremental removal patch)
5. Hard-remove `admission` module and rewrite affected boundaries in one pass.
6. Preserve deterministic settlement/accounting invariants in ledger reconciliation.

## 9) Open Decisions Requiring Your Approval Before L1
All previous L1 blockers are now resolved by user decisions.
Only explicit stage approval is pending: proceed to L1 or revise L0 further.

## 10) Working Assumptions (If Not Overridden)
1. Keep Spine as capability catalog owner for Body Endpoint capabilities.
2. Keep deterministic ledger settlement invariants.
3. `Act` does not include reservation/attribution linkage fields.
4. Pipeline contract is `Continue | Break` only.
5. Implement `sleep` as canonical shutdown sense emitted by `main` on SIGINT/SIGTERM.
6. Persist `CognitionState` goal stack in Continuity.
7. Sense queue uses bounded MPSC with blocking senders.
8. `new_capabilities` and `drop_capabilities` are canonical patch-based senses.
9. `Break` stops current `Act` only.

## 11) L0 Exit Criteria
L0 is complete when:
1. current-vs-target gaps are explicit,
2. architectural trade-offs are visible,
3. migration blast radius is mapped,
4. L1 decisions requiring your control are listed.

Status: `READY_FOR_L1_APPROVAL`
