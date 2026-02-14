# RESULT - core-cortex-act-stem-refactor

## Outcome

Implemented the runtime cutover from Admission-based flow to Stem+Act flow.

## Delivered Changes

1. Admission removal
- deleted `core/src/admission/*`
- removed `brainstem` runtime loop
- removed all compile-time `crate::admission` dependencies

2. New runtime contracts
- added `core/src/runtime_types.rs`
  - `Sense`, `Act`, `PhysicalState`, `CognitionState`
  - capability patch/drop payloads
  - dispatch decision enum
- added `core/src/ingress.rs`
  - bounded MPSC ingress wrapper
  - gate close + blocking sleep enqueue contract

3. Stem runtime loop
- added `core/src/stem.rs`
  - serial loop over sense queue
  - `sleep` interception (no Cortex call)
  - same-cycle capability patch/drop effect
  - physical state composition
  - serial dispatch pipeline: Ledger -> Continuity -> Spine
  - break semantics scoped to current act only

4. Main/runtime wiring
- rewired `core/src/main.rs`
  - queue creation and component startup
  - OS signal handling (SIGINT/SIGTERM)
  - shutdown sequence: close ingress gate -> blocking sleep enqueue -> await stem -> cleanup

5. Module refactor
- Cortex now exposes pure boundary contract and emits `Act[]`
- Spine dispatch contract migrated to single `ActDispatchRequest -> SpineEvent`
- Continuity refactored to cognition persistence + capability overlay + dispatch callback surface
- Ledger stage added for pre-dispatch reservation + settlement reconciliation
- Body endpoint adapters migrated to Act-based invocation
- Wire protocol removed `admission_feedback`, added capability patch/drop support

6. Config/schema updates
- removed obsolete loop batching fields
- retained `loop.sense_queue_capacity` as the single queue sizing control

7. Test migration
- removed obsolete admission test suites
- added/updated cortex, continuity, spine, stem BDT coverage
- added shutdown/backpressure and control-sense contract tests

## Verification

Executed:

1. `cargo check`
2. `cargo test cortex:: -- --nocapture`
3. `cargo test continuity:: -- --nocapture`
4. `cargo test spine:: -- --nocapture`
5. `cargo test ledger:: -- --nocapture`
6. `cargo test stem:: -- --nocapture`
7. `cargo test -- --nocapture`

All commands passed.
