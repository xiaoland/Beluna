# Drift Diagnosis: `_svc_v91.md` vs Current Beluna Framework

## Scope

- Candidate artifact: `docs/_svc_v91.md`
- Baseline framework anchors:
  - `AGENTS.md`
  - `docs/00-meta/index.md`
  - `docs/00-meta/concepts.md`
  - `docs/task/README.md`
  - `docs/10-prd/index.md`
  - `docs/20-product-tdd/index.md`
  - `docs/30-unit-tdd/index.md`
  - `docs/40-deployment/index.md`
- Method:
  - Diagnose only explicit V9.1 additions and rewritten claims in `_svc_v91.md`.
  - Do not treat placeholder sections such as "Content remains the same as V9" as independently specified requirements.

## Already Aligned Or Not Diagnosed As Drift

- Selective-memory posture is aligned: Beluna already states that documentation is not a parallel runtime.
- PRD ownership is aligned: product what/why and canonical product/domain semantics already belong to `docs/10-prd`.
- Mechanically enforced truth is aligned: code/tests/CI remain the implementation SSoT.
- Durable layer split is aligned: `10-prd`, `20-product-tdd`, `30-unit-tdd`, and `40-deployment` already match the candidate layer model.
- Hard-unit-first Unit TDD admission is aligned: Beluna already limits full Unit TDD packs to hard units.
- `docs/00-meta` is a compatible Beluna-local extension. `_svc_v91.md` not mentioning it is not, by itself, a contradiction.

## Material Drifts

### D1. Task Workspace Location Drift

- `_svc_v91.md` uses repo-root `tasks/` as the volatile workspace and repeats that location in the filesystem sketch, Mode A, anti-pattern text, and migration guidance.
- Beluna's current framework routes volatile work to `docs/task` and treats that path as a stable repo convention.
- Why this matters:
  - literal adoption would create two competing task homes,
  - current routing and promotion language would become internally inconsistent,
  - existing task packs would sit outside the candidate's declared workspace.
- Evidence:
  - `_svc_v91.md`: lines 75-91, 121, 169, 189, 203, 214
  - `AGENTS.md`: lines 13, 46, 62
  - `docs/00-meta/index.md`: line 18
  - `docs/00-meta/concepts.md`: line 11
  - `docs/task/README.md`: lines 1-3, 39-45

### D2. Task Admission And Ceremony Drift

- `_svc_v91.md` makes task-space usage mandatory for exploratory ambiguity and describes it as the only safe place for unstable reasoning.
- Beluna only opens a task folder when the work is large, ambiguous, or needs temporary coordination. Straightforward changes do not need task docs, and historical deep packs are explicitly not the default workflow.
- Why this matters:
  - the candidate increases documentation ceremony,
  - it weakens the current "record only the notes you actually need" rule,
  - it risks turning exploration into routine task-pack creation.
- Evidence:
  - `_svc_v91.md`: lines 117-123, 165-177, 187-203
  - `AGENTS.md`: lines 46, 62
  - `docs/task/README.md`: lines 10-20, 31-37, 47

### D3. Confirmation-Gated Execution Drift

- Mode B and Mode C require the agent to await human confirmation before updating durable docs or before writing tests/code.
- Beluna's current workflow does not insert a mandatory approval gate between restatement and execution. It says to pick the owner, read the smallest relevant slice, implement, and verify.
- Why this matters:
  - this changes the autonomy contract from guarded execution to explicit checkpoint workflow,
  - routine execution would slow even when scope is already clear,
  - the candidate shifts the framework toward a more waterfall-shaped handoff than Beluna currently uses.
- Evidence:
  - `_svc_v91.md`: lines 126-139
  - `AGENTS.md`: lines 20-31, 33-47

### D4. Dynamic Mode Protocol Drift

- `_svc_v91.md` promotes a mandatory A/B/C execution state machine as an AGENTS responsibility.
- Beluna currently has a lighter protocol: read the nearest relevant `AGENTS.md`, pick the governing layer, read the smallest relevant slice, and restate only for risky or reference-sensitive changes.
- Why this matters:
  - adopting v9.1 literally would require rewriting root and local `AGENTS.md` files,
  - the change is not just wording; it changes how agents dispatch work,
  - the new protocol overlaps with but is not reducible to the current restatement rule.
- Evidence:
  - `_svc_v91.md`: lines 93-141, 197-203
  - `AGENTS.md`: lines 18-47

### D5. Task Cleanup Ownership Drift

- `_svc_v91.md` tells the agent to ask the human whether the original task document can be deleted or archived.
- Beluna currently treats stale task detail as maintenance judgment: delete or ignore it once it stops being useful.
- Why this matters:
  - this is a small drift,
  - but it still adds a human checkpoint where the current framework uses local judgment.
- Evidence:
  - `_svc_v91.md`: lines 136-139
  - `docs/task/README.md`: line 47

## Conditional Or Non-Material Differences

### N1. Optional Alignment Substrate Is Not Currently A Drift

- `_svc_v91.md` mentions optional `15-alignment/`.
- Beluna currently has no `docs/15-alignment`, but the candidate also says alignment artifacts are justified only when repeated coordination drift exists.
- Diagnosis:
  - no current contradiction,
  - but if Beluna wants to adopt `15-alignment`, it must first define how that folder coexists with the existing optional `docs/00-meta` note.

### N2. Layer Ownership Model Remains Compatible

- `_svc_v91.md` keeps PRD, Product TDD, Unit TDD, and Deployment ownership boundaries that already match current Beluna indexes.
- Diagnosis:
  - no drift to fix here,
  - the main divergence is execution protocol and task handling, not layer ownership.

## Recommended Alignment Direction

If Beluna wants a Beluna-local v9.1 rather than literal adoption:

1. Replace every `tasks/` reference with `docs/task`.
2. Keep the ambiguity assessment idea, but do not make task creation mandatory for every exploratory prompt.
3. Keep the pre-execution restatement rule, but do not add a universal "await human confirmation" gate for Mode B or Mode C.
4. If an alignment substrate is introduced later, route it explicitly relative to `docs/00-meta` so Beluna does not create two meta owners.
