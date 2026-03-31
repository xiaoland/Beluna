# L0 Plan - Refactor Mind into Cortex + Non-Cortex
- Task Name: `refactor-mind-cortex-non-cortex`
- Stage: `L0` (request + context analysis only)
- Date: `2026-02-10`
## 1) Problem Deconstruction
The requested refactor is not a simple rename. It changes the control philosophy:
1. Cortex is deliberative and free in intent.
- It owns goal formation, decomposition, and pursuit.
- It may generate any intention, including defection/non-compliance/self-harm.
2. Continuity and safety must not depend on cortex internals.
- Cortex is replaceable and fallible.
- Survival/safety must be guaranteed by mechanism, not by "smart reasoning."
3. Constraints are runtime affordances, not memory/prompt narration.
- Bounds must be enforced mechanically during execution.
- Cortex experiences bounds as resistance/denial/degradation/cost.
4. Constraints are innate priors.
- Beluna should not depend on relearning core safety/survival constraints from scratch.
- Priors are encoded as affordance limits, cost gradients, and budget mechanics.
5. Intent freedom + execution constraint must coexist.
- Cortex can intend and attempt anything.
- Only constraint-compatible actions are effectuated.
6. Constraint semantics are physical/economic, not moral.
- No "obey policy because instructed" layer.
- Hard constraints: impossible actions.
- Economic constraints: capability loss, budget loss, energy/tool throttling.
- Soft constraints: friction/cost gradients.
7. Budget is survival.
- User-granted budget/capability should directly affect continued operation.
- Defection is not impossible in principle, but should be unsustainable in mechanism.
8. Non-cortex role is "world physics + economics."
- Non-cortex must not be a deliberative ruler.
- Non-cortex defines possible/costly/fundable/persistent action space.
9. Interface relations requested:
- Cortex <-> Non-cortex: constraints sensed through acting.
- Cortex <-> Spine: cortex emits intents; spine returns execution outcomes/events.
## 2) Context Collection (Sub-agent Style)
To reduce cognitive load, context was split into two focused tracks:
1. Sub-agent A: codebase cartography (parallel static scan)
- mapped `core/src/mind/*`, `core/tests/mind/*`, and contracts/docs.
- extracted deterministic invariants, loop ordering, and boundary rules.
2. Sub-agent B: architecture precedent scan (Firecrawl)
- collected enforcement-mechanism and constraint-economics references.
- focused on sources relevant to replaceable cognition + mechanical enforcement.
## 3) Current Codebase Reality
### 3.1 Mind today is a single mixed boundary
Current module: `core/src/mind/*`
`MindFacade::step` currently executes this deterministic order:
1. command ingestion + base state effects
2. preemption decision
3. delegation planning
4. evaluation
5. conflict resolution
6. memory policy
7. evolution decision
8. invariant assertion + typed output
The same module currently contains:
- deliberative logic (`preemption`, `evaluator`, `evolution`)
- lifecycle authority (`goal_manager`)
- continuity state (`MindState`)
- memory-policy hooks
### 3.2 Existing invariants and tests are strong
Current contracts/tests enforce:
- single active goal invariants
- closed preemption set (`pause|cancel|continue|merge`)
- deterministic conflict tie-breaks
- proposal-only evolution
- deterministic facade loop order
Key artifacts:
- contracts: `docs/contracts/mind/*`
- tests: `core/tests/mind/*`
### 3.3 Architectural gaps against requested model
- no `spine` concept exists yet in docs/code.
- no explicit "affordance physics/economics" execution layer.
- continuity is still coupled to deliberative state (`MindState` inside mind module).
- constraints are mostly policy semantics, not capability economics.
### 3.4 Terminology drift is already present
- docs mention future `Nexus` + `Cognition` framing in `docs/product/overview.md`.
- feature/module/contracts indexes still center `Mind`.
- no `docs/features/nexus/*` or `docs/features/cognition/*` exists yet.
Implication: this refactor needs an explicit terminology/migration strategy to avoid document/API fragmentation.
## 4) Invariant Fit-Gap Matrix
1. Cortex as highest deliberative authority
- Status: `PARTIAL`
- Reason: goals/evaluation/evolution already exist, but mixed with continuity/safety machinery.
2. Continuity/safety independent from cortex internals
- Status: `GAP`
- Reason: continuity and operational decisions are currently in the same module and same cycle logic.
3. Constraints as affordances, not memory narration
- Status: `GAP`
- Reason: no explicit affordance-enforcement substrate yet; memory policy is symbolic.
4. Innate constraints (priors)
- Status: `GAP`
- Reason: no first-class hard/economic/soft prior model in runtime.
5. Freedom of intent, constrained execution
- Status: `GAP`
- Reason: current command model does not distinguish attempted vs effectuated action with physics/economic denial reasons.
6. Physical/economic (not moral) constraints
- Status: `PARTIAL`
- Reason: `ai_gateway::budget` provides budget mechanics, but only for inference dispatch and not generalized execution affordances.
7. Budget as survival
- Status: `GAP`
- Reason: no unified survival budget across cortex action space.
8. Non-cortex as world, not ruler
- Status: `GAP`
- Reason: no dedicated non-deliberative world/economics layer exists yet.
9. Cortex <-> Spine intent/outcome protocol
- Status: `GAP`
- Reason: no spine module/contracts exist.
## 5) Architectural Trade-offs Identified
1. Rename-only vs semantic split
- Rename-only is low-cost but does not satisfy requested invariants.
- Semantic split requires deeper module and contract redesign.
2. Backward compatibility of `mind` API
- Keeping compatibility wrappers reduces breakage risk.
- Hard cut to new APIs reduces ambiguity but is riskier.
3. Where continuity state lives
- If continuity remains in cortex, replaceability invariant is violated.
- Moving continuity to non-cortex changes many interfaces but matches requirement.
4. Constraint representation granularity
- Coarse allow/deny is simpler but weak for "economic physics."
- Rich results (deny/degrade/cost) improve realism and debuggability.
5. Budget topology
- Isolated per-subsystem budgets are easier to ship.
- Unified survival budget is more aligned with requested behavior but requires cross-module accounting.
6. Testing strategy
- Rewriting tests by behavior contracts preserves confidence.
- Rewriting tests by implementation details risks regressions.
## 6) External Source Findings (Firecrawl)
These are architecture reality checks, not code truth.
1. Reference monitor mechanics (always-invoked, tamper-resistant, analyzable)
- Anderson report (1972): [Computer Security Technology Planning Study](https://csrc.nist.rip/publications/history/ande72.pdf)
- Cornell chapter: [Reference Monitors](https://www.cs.cornell.edu/fbs/publications/chptr.enfMech.refMonitor.pdf)
- Relevance: safety enforcement can be substrate-level and independent of principal internals.
2. Complete mediation + least privilege
- Saltzer & Schroeder: [The Protection of Information in Computer Systems](https://www.cs.virginia.edu/~evans/cs551/saltzer/)
- Relevance: enforcement should mediate all relevant actions, with attenuated authority by default.
3. Capability wrappers and robust safety under untrusted context
- OCPL paper: [Robust and compositional verification of object capability patterns](https://people.mpi-sws.org/~dreyer/papers/ocpl/paper.pdf)
- Relevance: authority attenuation/wrapping can preserve invariants even with adversarial surrounding code.
4. Budgeted control with explicit cost constraints
- BMDP/BRL: [Budgeted Reinforcement Learning in Continuous State Space](https://arxiv.org/abs/1903.01004)
- Relevance: action selection under explicit cost budget is a direct formal analogue for "budget is survival."
5. Affordances as action possibilities constrained by actor-environment coupling
- Review: [What is an affordance? 40 years later](https://pubmed.ncbi.nlm.nih.gov/28432011/)
- Relevance: supports encoding constraints as perceived action possibilities, not narrated rules.
## 7) L0 Recommendation (Scope Boundary for Next Stage)
For L1+ planning, recommend targeting a semantic split with compatibility migration:
1. Introduce new core boundaries:
- `core/src/cortex/*`: deliberative goal/intention generation.
- `core/src/non_cortex/*`: continuity, affordance physics, economics/budget, persistence.
- `core/src/spine/*` (or equivalent port): execution bridge returning outcome events.
2. Keep a transitional `mind` facade:
- map old API to new internals where feasible.
- preserve existing tests/contracts while adding new ones.
3. Add first-class attempted-vs-effectuated action model:
- cortex emits attempts/intents.
- non-cortex resolves each attempt into:
  - allowed + execution plan
  - denied (hard physics)
  - degraded/throttled (economic/soft constraints)
  - charged cost / budget delta
4. Preserve deterministic loop discipline.
5. Extend contracts before implementation rewrite:
- explicit non-cortex enforcement contracts
- budget-survival contracts
- cortex-spine IO contracts
## 8) Open Questions Requiring User Decision
1. Terminology authority
- Should `Cortex + Non-cortex` replace `Mind` as canonical immediately, or coexist via aliases (`Cognition/Nexus`) for one release?
2. Compatibility target
- Must `beluna::mind` remain source-compatible during this refactor?
3. Spine scope in this task
- Should we define `Spine` as contracts/interfaces only, or also integrate with runtime execution surfaces in MVP?
4. Defection modeling
- Do you want explicit intent variants for defection/self-harm/non-compliance in type contracts now, or keep intent taxonomy generic and model behavior through constraints?
5. Budget unification
- Should AI Gateway budget be folded into one non-cortex survival budget in this task, or remain separate with an adapter bridge first?
## 9) Working Assumptions (If Not Overridden)
1. This task performs a semantic split, not just renaming.
2. `mind` compatibility shims are kept for migration safety.
3. `Spine` starts as contract/interface boundary in this refactor.
4. Existing deterministic behavior contracts are preserved unless explicitly superseded.
5. Constraint enforcement is implemented as execution affordance outcomes, not prompt instructions.
## 10) Exit Criteria for L0
L0 is complete when:
- request is deconstructed into enforceable architecture properties,
- current code/doc reality is mapped,
- invariant fit-gaps are explicit,
- trade-offs are identified,
- external precedent is captured,
- gating questions for L1 are listed.
Status: `READY_FOR_L1_APPROVAL`
