# L3 Plan 08 - Docs and Contracts Refresh
- Task: `cortex-loop-architecture`
- Micro-task: `08-docs-contracts-refresh`
- Stage: `L3`
- Date: `2026-03-02`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Objective
Execute a deterministic docs/contracts refresh so repository documentation matches the implemented Cortex loop architecture and wire/runtime contracts.

## 2) Execution Steps
### Step 1 - Contract Docs (P0)
1. Update Cortex contract doc:
- output model (`emitted_acts + control`),
- per-act `wait_for_sense` integer semantics (`0` no wait),
- merged expand tool contract (`expand-senses`).
2. Update Stem contract doc:
- Stem emits tick grants and owns physical/efferent runtime responsibilities,
- remove control-sense orchestration and hibernate-loop claims.
3. Update Spine contract doc:
- descriptor/proprioception updates via direct `StemControlPort` runtime calls,
- NDJSON auth/sense/act_ack field contracts and endpoint-id canonicalization notes.
4. Update Continuity contract doc:
- cognition persistence + deterministic validation + `on_act` gate only.

### Step 2 - Module Docs (P1)
1. Refresh Cortex module docs/topology to current output schema and tool contracts.
2. Refresh Stem module docs/topology to split runtime responsibilities (tick runtime vs Cortex runtime).
3. Refresh Spine topology wire examples:
- `auth { endpoint_name, ns_descriptors }`
- `sense { payload, weight, act_instance_id? }`
- `act_ack { act_instance_id }`.
4. Refresh Body/Continuity module docs for current scope and payload contracts.
5. Refresh cross-module topology doc to remove obsolete flow and naming.

### Step 3 - Product/Operational Docs (P2)
1. Update `docs/overview.md` invariants and operational flow text.
2. Update `docs/glossary.md` terms:
- `EmittedAct`
- `wait_for_sense_seconds`
- `expected_fq_sense_ids`
- `expand-senses` sense-id format.
3. Update AGENTS docs where stale contract claims remain (`core`, `apple-universal`).
4. Update `tasks/cortex-loop-architecture/RESULT.md` with explicit delta summary for `07` and `08`.

### Step 4 - Consistency and Drift Sweep
1. Run stale-term scans and fix residual drift in targeted docs.
2. Confirm no contract contradictions between `docs/contracts/*` and `docs/modules/*`.
3. Confirm all examples use text payload + `weight` + optional `act_instance_id` where applicable.

## 3) File-Level Change Map
1. `docs/contracts/cortex/README.md`
2. `docs/contracts/stem/README.md`
3. `docs/contracts/spine/README.md`
4. `docs/contracts/continuity/README.md`
5. `docs/modules/cortex/README.md`
6. `docs/modules/cortex/TOPOGRAPHY.md`
7. `docs/modules/cortex/TOPOLOGY_ANALYSIS.md`
8. `docs/modules/stem/README.md`
9. `docs/modules/stem/TOPOGRAPHY.md`
10. `docs/modules/spine/TOPOGRAPHY.md`
11. `docs/modules/continuity/README.md`
12. `docs/modules/body/README.md`
13. `docs/modules/TOPOGRAPHY.md`
14. `docs/overview.md`
15. `docs/glossary.md`
16. `core/AGENTS.md`
17. `apple-universal/AGENTS.md`
18. `apple-universal/README.md`
19. `tasks/cortex-loop-architecture/RESULT.md`

## 4) Verification Gates
### Gate A - Stale Contract Term Sweep
```bash
rg -n "wait_for_sense: bool|is-wait-for-sense|expand-sense-raw|expand-sense-with-sub-agent|new_neural_signal_descriptors|drop_neural_signal_descriptors|\\bhibernate\\b|\\bmetadata\\b.*act_instance_id" docs/contracts docs/modules docs/overview.md docs/glossary.md core/AGENTS.md apple-universal/README.md apple-universal/AGENTS.md
```
Expected:
1. no stale contract phrasing in targeted refreshed docs.

### Gate B - NDJSON Wire Naming Consistency
```bash
rg -n "\"capabilities\"|endpoint_id.*sense|act_ack.*result|ns_descriptors|act_instance_id|weight" docs/contracts docs/modules apple-universal
```
Expected:
1. neural-signal auth naming uses `ns_descriptors`.
2. sense examples match current wire fields.
3. `act_ack` examples use `act_instance_id`.

### Gate C - Ownership Consistency
```bash
rg -n "Stem.*invok|Stem.*calls Cortex|control sense|Continuity.*capability overlay|Cortex.*stateless boundary" docs/contracts docs/modules docs/overview.md docs/glossary.md
```
Expected:
1. runtime ownership model is consistent with current architecture.

## 5) Completion Criteria (08)
1. P0 contract docs are accurate and internally consistent.
2. P1 module docs/topologies align with implemented runtime behavior.
3. P2 product/operational docs and task result are synchronized.
4. Stale-term sweeps for targeted drift patterns are clean.

Status: `READY_FOR_REVIEW`
