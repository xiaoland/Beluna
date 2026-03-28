# SVCv9 Document-System Drift Remediation Plan

## Plan Decomposition
- `L0-PLAN.md`: context, SSoT precedence, and V8->V9 delta map.
- `L1-PLAN.md`: strategy and decision model.
- `L2-PLAN-01-policy-target-state.md`: target policy contracts.
- `L2-PLAN-02-file-change-packages.md`: file-level change packages.
- `L2-PLAN-03-unit-tdd-admission-audit.md`: hard-unit admission audit and profile recommendation.
- `L3-PLAN.md`: execution order, rollback, and exit criteria.

## Perturbation
Request to indicate current drift between Beluna document system and `SVCv9` baseline, then propose an actionable remediation plan in `docs/task`.

## Input Type
Primary: `Artifact`
Secondary: `Intent`

## Governing Anchors
- `docs/00-meta/index.md`
- `docs/00-meta/read-order.md`
- `docs/00-meta/doc-system.md`
- `docs/00-meta/concepts.md`
- `docs/00-meta/promotion-rules.md`
- `docs/10-prd/index.md`
- `docs/20-product-tdd/index.md`
- `docs/30-unit-tdd/index.md`
- `AGENTS.md`
- `core/AGENTS.md`
- `apple-universal/AGENTS.md`
- `/Users/lanzhijiang/Downloads/svc_v_9_with_alignment.md`
- `/Users/lanzhijiang/Library/Mobile Documents/com~apple~CloudDocs/sustainable_vibe_coding_framework_v_8.md` (context only; non-normative)

## Intended Change
Define and execute a minimal-risk migration path that reduces material drift with SVCv9 while preserving Beluna's existing useful constraints.

## Impact Hypothesis
- Primary hit: `docs/00-meta` and root/component `AGENTS.md` governance posture.
- Secondary hits: `docs/10-prd` terminology ownership, `docs/30-unit-tdd` admission rules, `docs/20-product-tdd` and deployment placement boundaries.
- Confidence: High (drift is mostly policy/ownership, not code behavior).
- Unknowns:
  - Whether Beluna intentionally wants stricter process than SVCv9 minimal baseline.
  - Whether current Unit TDD coverage (`core`, `apple-universal`, `cli`, `monitor`) should stay at full-pack depth for all units.

## Temporary Assumptions
- Keep current folder topology (`00-meta`, `10`, `20`, `30`, `40`) unless explicit simplification is requested.
- Prefer policy edits over file-system churn for first pass.
- Do not rewrite historical task artifacts.

## Negotiation Triggers
- If we choose strict SVCv9 alignment, some current governance rigidity will be reduced.
- If we keep universal Unit TDD coverage, drift against SVCv9 remains by design.
- If we retain large operational snapshots in component `AGENTS.md`, AGENTS drift risk remains.

## Acceptance Criteria
1. Each current drift item has one explicit disposition: `align_now`, `align_later`, or `keep_intentional_drift`.
2. Canonical product/domain semantics ownership is explicit and non-conflicting.
3. Read protocol language is changed from mandatory/ritual to guidance-by-context.
4. Unit TDD policy explicitly states admission rationale (hard-unit-only vs all-units-by-policy).
5. AGENTS files have a clear boundary between durable guardrails and mutable runtime snapshot content.
6. Promotion policy includes demotion/removal criteria for stale durable docs.
7. No authoritative truth is duplicated across layers after edits.

## Guardrails Touched
- Documentation consistency review only (no runtime code or schema change).
- Cross-layer ownership checks (manual).

## Evidence Expected
- Diff of all changed docs.
- Post-edit drift checklist showing closure status per item.
- One short rationale note for every intentional remaining drift.

## Outcome
`promote`

## Promotion Candidates
- Stable policy on canonical semantics ownership.
- Stable policy on Unit TDD admission strategy.
- Stable AGENTS sizing/scope boundary rule.
- Stable demotion rule for durable docs.

---

## Baseline Drift Register (from assessment)

1. `00-meta` treated as mandatory constitutional layer vs SVCv9 optional baseline.
2. Cross-layer read order has ritual/must-read shape vs SVCv9 contextual read strategy.
3. Canonical terminology ownership in `00-meta/concepts.md` vs SVCv9 PRD ownership.
4. Unit TDD required for every unit with fixed six-doc template vs SVCv9 hard-unit admission.
5. Component `AGENTS.md` include large mutable “Current State” snapshots (high drift potential).
6. Promotion policy lacks explicit demotion/removal lifecycle language.

---

## Execution Plan

## Phase 0 - Policy Decision Lock
Goal: avoid editing churn before alignment strategy is explicit.

Steps:
1. Decide target mode per drift item:
- `strict_svcv9`: align with SVCv9 defaults unless Beluna-specific pressure proves otherwise.
- `beluna_strict`: keep stricter governance but document each intentional drift.
2. Record decision table in this task folder before making authority-layer edits.

Deliverable:
- `DECISIONS.md` in this task folder.

## Phase 1 - Ownership and Read-Path Normalization
Goal: resolve highest-impact governance drift without structural deletion.

Steps:
1. Update `docs/00-meta/index.md` and `docs/00-meta/read-order.md` wording:
- Keep guidance, remove mandatory/ritual implications.
- Preserve “smallest relevant slice” behavior.
2. Update `docs/00-meta/promotion-rules.md` and `docs/00-meta/doc-system.md`:
- Clarify canonical product/domain semantics owner.
- Add explicit demotion/removal rule for stale durable docs.
3. If adopting PRD-owned semantics strictly:
- Add/refresh PRD glossary home and make `00-meta/concepts.md` a cross-layer operational ontology only.

Deliverables:
- Updated `00-meta` docs.
- If needed, new PRD glossary artifact and link from `docs/10-prd/index.md`.

## Phase 2 - Unit TDD Admission Policy Reconciliation
Goal: remove ambiguity between SVCv9 “hard-unit-only” and Beluna “all-units” stance.

Steps:
1. Update `docs/30-unit-tdd/index.md` with explicit admission policy:
- Option A: hard-unit-only admission (SVCv9 strict).
- Option B: all-units required by Beluna policy (intentional drift, documented rationale).
2. Ensure `docs/20-product-tdd/index.md` and `docs/00-meta/doc-system.md` use consistent language.

Deliverables:
- Synchronized policy text in `30`, `20`, and `00-meta`.

## Phase 3 - AGENTS Scope Slimming
Goal: keep AGENTS practical and low-drift.

Steps:
1. Root `AGENTS.md`:
- Keep operating constraints and navigation.
- Add concise pre-execution restatement protocol for risky/reference-sensitive changes.
2. `core/AGENTS.md` and `apple-universal/AGENTS.md`:
- Move volatile capability snapshots to task/result artifacts or dated release notes.
- Keep only durable constraints, boundaries, and high-risk warnings.

Deliverables:
- Slimmed AGENTS files with reduced volatile content.

## Phase 4 - Validation and Promotion Closure
Goal: close task with explicit drift status and authoritative consistency proof.

Steps:
1. Build a final drift checklist with status per item:
- `resolved`
- `intentional_drift`
- `deferred_with_reason`
2. Verify no duplicate ownership statements across `00/10/20/30/40`.
3. Produce `RESULT.md` and include promotion notes.

Deliverables:
- `RESULT.md` in this task folder.
- Optional follow-up backlog entry for deferred drifts.

---

## Work Breakdown (Recommended PR Slices)

1. PR-1: Phase 1 (ownership/read-path + demotion rule).
2. PR-2: Phase 2 (Unit TDD admission reconciliation).
3. PR-3: Phase 3 (AGENTS slimming + restatement protocol).
4. PR-4: Phase 4 (validation + final result packet).

Each PR should keep doc ownership changes internally consistent and avoid partial-policy states.
