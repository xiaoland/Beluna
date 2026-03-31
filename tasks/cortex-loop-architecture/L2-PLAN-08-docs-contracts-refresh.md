# L2 Plan 08 - Docs and Contracts Refresh
- Task: `cortex-loop-architecture`
- Micro-task: `08-docs-contracts-refresh`
- Stage: `L2`
- Date: `2026-03-02`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Goal and Scope
Goal:
1. Refresh contract and module docs to match the implemented Cortex loop architecture after micro-tasks `01..07`.
2. Remove stale terminology/flows (`capabilities` in neural-signal domain, control-sense loop ownership, bool `wait_for_sense`).
3. Publish deterministic canonical wording/snippets reused across docs to reduce drift.

In scope:
1. Contract docs (`docs/contracts/*`) for Cortex/Stem/Spine/Continuity.
2. Module docs/topologies (`docs/modules/*`) impacted by loop/pathway/state changes.
3. High-level docs (`docs/overview.md`, `docs/glossary.md`, AGENTS impacted sections).
4. Task result summary (`tasks/cortex-loop-architecture/RESULT.md`).

Out of scope:
1. New runtime behavior implementation.
2. Retrofitting all legacy historical task artifacts beyond required task result updates.

## 2) Canonical Contract Blocks (Single-Source Wording)
1. NDJSON `auth` contract:
- `endpoint_name`
- `ns_descriptors`
- optional `proprioceptions`.
2. NDJSON `sense` contract:
- `sense_instance_id`
- `neural_signal_descriptor_id`
- `payload` (text)
- `weight` (`[0,1]`)
- optional `act_instance_id`.
3. Runtime ownership contract:
- Stem emits ticks and owns physical-state mutation + pathway construction.
- Cortex runtime owns cycle execution and afferent consumption.
4. Act emission contract:
- Primary emits dedicated act tools with per-act `wait_for_sense` integer (`0` means no wait).
5. Sense expansion contract:
- single tool `expand-senses`
- `mode: raw | sub-agent`
- `senses_to_expand[].sense_id` formatted as `"<monotonic-id>. <fq-sense-id>"`.

## 3) Document Inventory Freeze
Priority P0 (contracts):
1. `docs/contracts/cortex/README.md`
2. `docs/contracts/stem/README.md`
3. `docs/contracts/spine/README.md`
4. `docs/contracts/continuity/README.md`

Priority P1 (module docs):
1. `docs/modules/cortex/README.md`
2. `docs/modules/cortex/TOPOGRAPHY.md`
3. `docs/modules/cortex/TOPOLOGY_ANALYSIS.md`
4. `docs/modules/stem/README.md`
5. `docs/modules/stem/TOPOGRAPHY.md`
6. `docs/modules/spine/TOPOGRAPHY.md`
7. `docs/modules/continuity/README.md`
8. `docs/modules/body/README.md`
9. `docs/modules/TOPOGRAPHY.md`

Priority P2 (overview/glossary/agents/result):
1. `docs/overview.md`
2. `docs/glossary.md`
3. `core/AGENTS.md`
4. `apple-universal/AGENTS.md`
5. `apple-universal/README.md`
6. `tasks/cortex-loop-architecture/RESULT.md`

## 4) Drift Rules Freeze
1. Neural-signal domain uses `ns_descriptor` naming; do not reintroduce `capabilities` for auth/schema in this domain.
2. Do not describe Stem as invoking Cortex directly.
3. Do not describe control-sense pipeline for descriptor/proprioception updates.
4. Do not document `wait_for_sense: bool`; use per-act integer seconds.
5. Do not document split expand tools (`expand-sense-raw`, `expand-sense-with-sub-agent`).
6. Do not describe sense `metadata` object field; use explicit fields and rendered metadata list format in Primary input text.

## 5) Deterministic Update Strategy Freeze
1. Update P0 contract docs first.
2. Propagate exact canonical blocks into P1 module docs.
3. Update P2 overview/glossary/AGENTS and task result last.
4. Run stale-term sweeps between each phase to avoid regressions.

## 6) Validation Gate Freeze
Required stale-term sweep set:
1. `wait_for_sense: bool`, `is-wait-for-sense`
2. `expand-sense-raw`, `expand-sense-with-sub-agent`
3. `new_neural_signal_descriptors`, `drop_neural_signal_descriptors` as control-sense flow claims
4. `hibernate` loop-control claims in current runtime docs
5. `auth` payload naming drift (`capabilities`, `descriptors`) in NDJSON wire examples
6. sense `metadata` field claims for correlated act linkage

## 7) Risks and Constraints
1. Risk: topology docs are broad and easy to partially update.
Mitigation: enforce phased update + sweep gates + focused canonical snippets.
2. Risk: conflating AI Gateway backend capabilities with neural-signal descriptor naming.
Mitigation: keep rename scope explicit to neural-signal/runtime docs only.
3. Risk: docs overfit transient internals.
Mitigation: contracts must describe stable external behavior, not temporary helper implementation details.

## 8) L2 Exit Criteria (08)
1. Canonical wording blocks are defined and reusable.
2. P0/P1/P2 update inventory is frozen and prioritized.
3. Drift rules and validation sweeps are explicit.
4. L3 can execute updates deterministically without scope ambiguity.

Status: `READY_FOR_REVIEW`
