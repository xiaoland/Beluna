# L0 Plan - Cortex Autonomy Refactor
- Task Name: `cortex-autonomy-refactor`
- Stage: `L0` (request + context analysis only)
- Date: `2026-02-21`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Problem Deconstruction
This task is a structural rewrite of Beluna Core cognition runtime, not a local patch.

Locked requirements from user input:
1. Stem progression must be tick/interval driven (configurable), not blocked on new sense arrival.
2. Continuity becomes centralized cognition store + deterministic guardrail authority.
3. Continuity must store and govern:
- `goal-tree` (replace `goal-stack`)
- `l1-memory`
4. Dispatch path must chain `Continuity`, `Ledger`, `Spine`; all three can emit `Sense`.
5. `goal-tree` must have two partitions:
- Root Partition: immutable, compile-time fixed, non-modifiable by Cortex.
- User Partition: mutable weighted tree, runtime-evolving via sprouting/pruning/tilting.
6. Cortex boundary becomes:
- input: `[senses, physical_state(act_descriptor_catalog), cognition_state(goal-tree, l1_memory)]`
- output: `acts, cognition_state`
7. Cortex IR pipeline remains helper-mediated but changes contracts:
- InputIR includes `senses`, `act-descriptor-catalog`, `goal-tree`, `l1-memory`.
- OutputIR includes `acts`, `goal-tree-patch`, `l1-memory-patch`.
8. IR format: first-level XML enforced; inner content can be Markdown (structure-first, no decorative formatting).
9. Primary prompt/invariants must enforce:
- per-tick amnesia model,
- cognition-state vs physical-state dualism,
- no social-performance text,
- act/goal-memory change-oriented closure,
- proactive intervention and self-evolution.
10. Documentation updates are mandatory: feature docs, module docs, `core/src/cortex/AGENTS.md`.

## 2) Context Collection (Sub-agent Style)
To reduce cognitive load, analysis was split into parallel tracks (virtual sub-agents):
1. Runtime loop track: `core/src/main.rs`, `core/src/stem.rs`, queue/shutdown/sleep semantics.
2. Cognition-state track: `core/src/types.rs`, `core/src/continuity/*`, cognition persistence and guardrails.
3. Cortex IR/prompt track: `core/src/cortex/*`, helper pipeline, IR contracts, tests.
4. Boundary/docs track: `docs/features/*`, `docs/modules/*`, `docs/contracts/*`, AGENTS files.
5. External reference track: tick scheduling, bounded queues, structured model output contracts.

## 3) Current Codebase Reality

### 3.1 Stem is sense-driven today
1. `Stem::run` blocks on `sense_rx.recv().await`, then drains queued senses in same cycle.
2. No timer/interval tick source exists.
3. `Sense::Sleep` is a hard stop signal; it breaks loop and skips Cortex.
4. `main` shutdown path explicitly enqueues `Sleep` sense.

### 3.2 Continuity is not yet centralized cognition authority
1. `CognitionState` currently is:
- `revision`
- `goal_stack: Vec<GoalFrame>`
2. No `goal-tree`, no `l1-memory`.
3. Continuity pre-dispatch guard currently always `Continue`.
4. Continuity records Spine events but does not enforce deterministic cognition guardrails beyond simple snapshot persist.

### 3.3 Cortex IR and helpers are goal-stack based
1. Input IR sections: `<senses>`, `<act-descriptor-catalog>`, `<goal-stack>`, `<context>`.
2. Output IR sections: `<acts>`, `<goal-stack-patch>`.
3. Output helper pair is `acts_helper` + `goal_stack_helper`.
4. Patch algebra is stack ops (`push/pop/replace_top/clear`), not tree operations.

### 3.4 Dispatch chain and sense emission today
1. Current act dispatch order in Stem: `Ledger -> Continuity -> Spine`.
2. Spine can indirectly produce senses (body endpoint inbound + capability patch/drop).
3. Continuity/Ledger do not currently emit senses back into afferent pathway.

### 3.5 Config and contract mismatch
1. `loop` config only has `sense_queue_capacity`; no tick interval/tick policy.
2. Docs/contracts are aligned to goal-stack and sense-driven progression.
3. `core/src/cortex/AGENTS.md` explicitly states `Progression is input-event driven only` (conflicts with new autonomy requirement).

## 4) Fit-Gap Matrix
1. Tick-driven autonomous Stem: `GAP`
2. Continuity as centralized guardrail + goal-tree/l1-memory store: `GAP`
3. Root/User partitioned goal-tree model: `GAP`
4. User partition weighted tree operations (sprout/prune/tilt): `GAP`
5. Output patches: goal-tree + l1-memory: `GAP`
6. Dispatch chain with tri-module sense emission: `PARTIAL` (chain exists but order and emission authority not aligned)
7. IR contract shift (`goal-stack` -> `goal-tree`, add `l1-memory`): `GAP`
8. Prompt invariants update for amnesia/teleology/no-performative-text: `GAP`
9. Docs synchronization across features/modules/agents: `GAP`

## 5) Architectural Trade-offs Identified
1. Tick engine policy:
- Option A: strict periodic `tokio::time::interval` with missed-tick burst behavior controlled.
- Option B: one-cycle-at-a-time scheduler with drift-tolerant sleep.
- Trade-off: determinism and cadence guarantees vs overload behavior and burst risk.

2. Sleep semantics under tick-driven runtime:
- Existing `Sleep` means stop loop.
- User note defines sleep as pause-until-new-sense.
- Trade-off: keep old shutdown contract vs redefine sleep as runtime mode transition.

3. Dispatch stage order:
- Existing is `Ledger -> Continuity -> Spine`.
- Request states Continuity/Ledger/Spine chain but not explicit order.
- Trade-off: pre-cost gating first (Ledger-first) vs cognition guardrail first (Continuity-first).

4. Deterministic goal-tree patch language:
- Minimal ops easier to hard-guard.
- Rich ops increase expressiveness but raise nondeterminism risk.

5. Root partition source:
- compile-time constants in Rust (strong immutability)
- embedded file at build-time (editable with code review)
- Trade-off: rigidity vs maintainability.

6. L1-memory model:
- append-only statements (simple replay)
- keyed/upserted statements (compact but needs merge rules)
- Trade-off: determinism simplicity vs memory growth.

7. Continuity as single writer:
- centralizes guardrails and deterministic patch application.
- increases module responsibility and coupling with Cortex contracts.

## 6) External Source Findings (Fallback from Firecrawl)
Note: Firecrawl was attempted per workspace rule but blocked by insufficient credits; fallback used direct primary docs.

1. Tokio interval periodic scheduling and missed tick behavior.
- Source: [docs.rs tokio::time::interval](https://docs.rs/tokio/latest/tokio/time/fn.interval.html)
- Relevance: informs deterministic tick loop behavior and overload handling policy.

2. Tokio bounded MPSC queue semantics and clean shutdown.
- Source: [docs.rs tokio::sync::mpsc](https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html)
- Relevance: supports preserving bounded afferent pathway while decoupling tick progression from sense arrival.

3. Structured output with schema-constrained extraction.
- Source: [OpenAI Structured Outputs](https://platform.openai.com/docs/guides/structured-outputs)
- Relevance: aligns with keeping Output helpers deterministic and schema-constrained for `goal-tree-patch` and `l1-memory-patch`.

## 7) L0 Recommendation for L1
L1 should lock architecture around three hard pivots:
1. Runtime pivot: introduce configurable tick loop and redefine sleep/hibernate control contract.
2. State pivot: replace `goal_stack` with partitioned `goal_tree` + `l1_memory`, with Continuity as deterministic patch guard.
3. Cognition pivot: keep helper-based pipeline but migrate IR/output patch contracts and prompt invariants for amnesia + teleology.

## 8) Decisions Needed From You Before L1
1. Tick cadence default and overload policy:
- choose interval unit/value default (e.g. 250ms/500ms/1000ms)
- choose missed tick handling (burst catch-up vs skip-to-latest)

2. Sleep semantics finalization:
- A) keep `Sleep=stop loop` and add new `Pause` sense
- B) redefine `Sleep=pause-until-new-sense`, add `Hibernate` for stop

3. Dispatch stage order:
- A) keep `Ledger -> Continuity -> Spine`
- B) switch to `Continuity -> Ledger -> Spine`

4. Goal-tree patch v1 operation set:
- minimum recommended: `sprout`, `prune`, `tilt`, `set_root_weight?` (or forbid root edits entirely)

5. L1-memory patch v1 model:
- A) append-only statements with bounded ring buffer
- B) keyed notes with deterministic upsert/delete operations

6. Root partition source format:
- A) Rust compile-time constants
- B) compile-time embedded markdown/xml file under `core/src/cortex/`

## 9) Working Assumptions (If You Don’t Override in L1)
1. Keep bounded sense queue semantics unchanged.
2. Keep helper pipeline structure (`input helpers -> primary -> output helpers`).
3. Replace goal-stack contracts completely (no dual-mode compatibility layer).
4. Continuity applies patches as deterministic single authority and rejects illegal root edits.
5. Introduce explicit tick config under `loop` with safe default.

## 10) L0 Exit Criteria
L0 is complete when:
1. target-vs-current gaps are explicit,
2. dispatch/state/IR trade-offs are enumerated,
3. external references back critical runtime decisions,
4. user-controlled decisions for L1 are isolated and ready for approval.

Status: `READY_FOR_L1_APPROVAL`
