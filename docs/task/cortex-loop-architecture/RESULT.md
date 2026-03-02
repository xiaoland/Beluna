# Cortex Loop Architecture - Implementation Result

Date: 2026-03-02

## Micro-task 02 - Sense Model and Wire Migration
### Scope implemented
1. Hard-cut `Sense` model in `core` from enum/`SenseDatum` to a single `Sense` struct.
2. Removed control-sense variants (`Hibernate`, `New/Drop NeuralSignalDescriptors`, `New/Drop Proprioceptions`) from runtime code paths.
3. Migrated sense payload contract to text-first wire (`payload: String`, `weight`, optional `act_instance_id`).
4. Replaced adapter control-sense emission with runtime control calls (`Spine adapters -> Spine runtime -> Stem control port`).
5. Added Stem-owned physical-state control store for descriptor/proprioception mutation and snapshots.

### Build verification
1. `cd core && cargo build` ✅
2. `cd cli && cargo build` ✅

## Micro-task 03 - Afferent Deferral Engine and Sidecar
### Scope implemented
1. Moved afferent pathway under Stem ownership:
   - removed `core/src/afferent_pathway.rs`
   - added `core/src/stem/afferent_pathway.rs`
   - rewired imports/exports to `stem::afferent_pathway`.
2. Implemented pathway-owned rule-control port:
   - `overwrite_rule` (single-rule upsert by `rule_id`)
   - `reset_rules` (clear all rules)
   - `snapshot_rules`.
3. Implemented deterministic deferral scheduler:
   - ingress MPSC -> rule match -> defer or forward.
   - deferral condition for `min_weight`: defer when `sense.weight < min_weight`.
   - regex selector against `fq_sense_id = endpoint_id/neural_signal_descriptor_id`.
4. Implemented deferred FIFO buffer management:
   - deferred entries remain buffered until rule update unblocks them.
   - `max_deferring_nums` cap enforced.
   - overflow evicts oldest deferred entries with warning log.
5. Implemented observe-only sidecar stream:
   - non-blocking broadcast events for overwrite/reset/deferred/released/evicted.
6. Added loop config knobs:
   - `loop.max_deferring_nums`
   - `loop.afferent_sidecar_capacity`
   - updated `core/beluna.schema.json` accordingly.

### Build verification
1. `cd core && cargo build` ✅
2. `cd cli && cargo build` ✅

## Micro-task 04 - Cortex Primary Tooling and Act Emission
### Scope implemented
1. Replaced prompt-text act dispatch with tool-call-native act emission in Cortex Primary.
2. Added dynamic per-act tool generation with transport-safe alias mapping:
   - alias name (for backend compatibility)
   - deterministic runtime mapping to fq act id (`endpoint_id/neural_signal_descriptor_id`).
3. Added per-act `wait_for_sense` integer semantics (bounded by `max_waiting_seconds`, `0` means no wait).
4. Added Primary static control tools:
   - `expand-senses`
   - `overwrite-sense-deferral-rule`
   - `reset-sense-deferral-rules`
   - `sleep`
   - `patch-goal-forest`
5. Merged old sense expansion tools into `expand-senses` with:
   - `mode: raw | sub-agent`
   - `senses_to_expand[]`
   - `sense_id` as composite reference id (`{monotonic_internal_sense_id}. {fq-sense-id}`).
6. Updated sense rendering delivered to Primary to deterministic line format:
   - `- [monotonic internal sense id]. [fq-sense-id]: [key=value,...]; [payload-truncated-if-needed]`
7. Refactored Cortex output contract:
   - from `Act[] + wait_for_sense(bool)`
   - to `emitted_acts[] + control directives`.
8. Implemented runtime wait path using afferent deferral-rule overwrite/clear flow (bounded timeout), with optional matching by `act_instance_id` and descriptor-declared emitted fq sense ids.
9. Removed dependence on `<somatic-acts>` and `<is-wait-for-sense>` output parsing for dispatch decisions.
10. Added `ReactionLimits.max_waiting_seconds` and updated config schema.

### Build verification
1. `cd core && cargo build` ✅
2. `cd cli && cargo build` ✅

## Micro-task 05 - Goal Forest Reset and Thread Rewrite
### Scope implemented
1. Extended `patch-goal-forest` tool contract to strict object input:
   - `patch_instructions: string`
   - `reset_context: bool` (default false).
2. Added AI Gateway thread-level generic atomic message mutation API:
   - `Thread::mutate_messages_atomically(ThreadMessageMutationRequest)`.
   - domain-agnostic selectors for trim boundaries (`FirstUserMessage`, `LatestAssistantToolBatchEnd`).
   - atomic trim + system-prompt update in one store write lock.
3. Added thread-scoped system prompt mode in AI Gateway store:
   - inherit chat default
   - override with replacement prompt
   - explicit clear.
4. Updated turn preparation to snapshot effective system prompt from thread state (not only chat default).
5. Wired Cortex Primary reset path:
   - apply goal-forest patch first.
   - when `reset_context=true`, call `mutate_messages_atomically` with:
     - trim range: first user message -> latest assistant tool batch end.
     - system prompt update: replace with prompt including updated goal-forest section.
6. Added primary micro-loop restart behavior after successful reset:
   - discard current tool-message follow-up chain.
   - start next internal step with fresh user message on same persistent thread.
7. Kept sprout numbering generation deterministic in downstream cognition patch stage and made it explicit via `resolve_sprout_numbering`.

### Build verification
1. `cd core && cargo build` ✅
2. `cd cli && cargo build` ✅

## Micro-task 06 - State Ownership and Continuity Refactor
### Scope implemented
1. Physical state ownership hardened in Stem:
   - canonical state is `Arc<RwLock<PhysicalState>>`
   - Stem write path only via `StemControlPort`.
2. `PhysicalState.capabilities` was renamed to `PhysicalState.ns_descriptor`.
3. Cortex physical-state reads were aligned to Stem snapshot path.
4. Cognition model hard-cut:
   - removed `l1_memory`
   - removed focal-awareness IR sections and helper flows.
5. Continuity scope narrowed:
   - removed descriptor overlay state and APIs
   - kept cognition persistence + validation + `on_act` gate.
6. Config/schema cleanup:
   - removed L1-memory-specific limits/helper routes.

### Build verification
1. `cd core && cargo build` ✅
2. `cd cli && cargo build` ✅

## Micro-task 07 - Efferent FIFO Serial Pipeline
### Scope implemented
1. Extracted efferent pipeline into dedicated module:
   - added `core/src/stem/efferent_pathway.rs`
   - updated `core/src/stem.rs` exports.
2. Preserved serial consumer order `Continuity -> Spine`.
3. Fixed per-cycle sequencing:
   - envelope field is now `act_seq_no`
   - sequence is assigned in Cortex runtime and is monotonic per cycle.
4. Producer API changed to explicit single-envelope enqueue:
   - `ActProducerHandle::enqueue(EfferentActEnvelope)`.
5. Added bounded shutdown drain behavior:
   - `loop.efferent_shutdown_drain_timeout_ms`
   - drain mode processes queued acts until deadline
   - on timeout, remaining queue entries are dropped with warning telemetry.
6. Updated runtime wiring and schema:
   - `main.rs` passes drain timeout to efferent runtime
   - `config.rs` + `beluna.schema.json` updated.

### Build verification
1. `cd core && cargo build` ✅
2. `cd cli && cargo build` ✅

## Micro-task 08 - Docs and Contracts Refresh
### Scope implemented
1. Refreshed contract docs to current runtime model:
   - `docs/contracts/cortex/README.md`
   - `docs/contracts/stem/README.md`
   - `docs/contracts/spine/README.md`
   - `docs/contracts/continuity/README.md`.
2. Refreshed module docs to current ownership and wire contracts:
   - `docs/modules/cortex/{README,TOPOGRAPHY,TOPOLOGY_ANALYSIS}.md`
   - `docs/modules/stem/{README,TOPOGRAPHY}.md`
   - `docs/modules/spine/TOPOGRAPHY.md`
   - `docs/modules/continuity/README.md`
   - `docs/modules/body/README.md`
   - `docs/modules/TOPOGRAPHY.md`.
3. Refreshed product/ops docs:
   - `docs/overview.md`
   - `docs/glossary.md`
   - `core/AGENTS.md`
   - `apple-universal/{README,AGENTS}.md`.
4. Locked canonical docs wording for:
   - `ns_descriptors` auth field
   - text payload + `weight` + optional `act_instance_id`
   - per-act integer `wait_for_sense` semantics
   - unified `expand-senses` tool contract.

## Notes
1. No backward compatibility window was introduced.
2. Existing unrelated workspace changes were preserved.
