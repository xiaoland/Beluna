# L1 Plan - Refactor Mind into Cortex + Non-Cortex (High-Level Strategy)
- Task Name: `refactor-mind-cortex-non-cortex`
- Stage: `L1` (high-level strategy)
- Date: `2026-02-10`
- Status: `DRAFT_FOR_APPROVAL`

## 0) Inputs Locked From L0 Approval
User decisions applied as hard constraints:
1. `Cortex + Non-cortex` is canonical now.
2. `beluna::mind` source compatibility is not required.
3. `Spine` scope is interface/contracts only for this task.
4. Spine definition (locked): always-on control substrate between Non-cortex and Body Endpoints; it carries intent-to-motion and feedback-to-cognition flow without requiring thought.
5. Non-cortex must never classify intents semantically; otherwise it becomes moral interpreter/shadow cortex.
6. Build one global survival budget ledger; initially feed it with partial/approximate debits from AI Gateway and improve fidelity later.

## 1) Strategy Summary
Replace `mind` with three explicit boundaries:
1. `cortex`: deliberative authority (goals, decomposition, intention generation).
2. `non_cortex`: continuity + affordance physics/economics + survival budget mechanics.
3. `spine`: execution substrate contract between non-cortex and body endpoints (contracts only in this phase).

Core strategy:
- perform a semantic split, not a rename.
- move continuity/safety mechanism ownership to non-cortex.
- model cortex outputs as attempts/intents; non-cortex resolves what is physically/economically effectuable.
- make budget/survival first-class at non-cortex level with global ledger accounting.
- preserve deterministic orchestration and contract-first development style.

## 2) Target High-Level Architecture

```text
Cortex (deliberation, free intent)
  -> emits IntentAttempt[]
      -> Non-cortex (mechanism)
         - continuity kernel
         - affordance engine (hard/economic/soft constraints)
         - survival budget ledger (global)
         - execution admissibility resolver
         -> Spine Port (always-on control substrate contract)
            -> Body Endpoints (future concrete adapters)
            <- Execution feedback/events
      <- Non-cortex outcome events (allowed/denied/degraded/costed)
<- Cortex consumes outcomes and updates plans/goals
```

Boundary meaning:
- Cortex owns "what to try."
- Non-cortex owns "what can happen and at what cost."
- Spine owns "how admitted actions flow to body endpoints and feedback returns."

## 3) Responsibility Split (Authoritative)

1. Cortex owns:
- goal formation/management/decomposition,
- intention/attempt generation,
- helper orchestration as cognition organs,
- deliberative reflection on outcomes.

2. Non-cortex owns:
- continuity identity/state not tied to cortex internals,
- constraint mechanics as affordances (no prompt narration),
- survival budget ledger and debit/credit accounting,
- effectuation gate: permit/deny/degrade/charge outcomes,
- persistence of operational state needed for survival continuity.

3. Spine owns (this phase: contracts only):
- accepted action handoff to body endpoint interfaces,
- feedback event return path from body endpoints,
- always-on transport/control semantics for execution flow.

## 4) Non-Cortex Non-Interpretation Rule

The non-cortex layer must not read or infer intent meaning.

Allowed:
- evaluate declared action affordance keys/capability handles,
- enforce hard impossibility constraints,
- apply economic costs, friction, throttles, degradation,
- return typed mechanical outcomes.

Disallowed:
- moral/policy narration ("this intent is bad"),
- semantic intent labeling ("defection intent", "self-harm intent"),
- normative judgment over cortex intent content.

Design implication:
- non-cortex APIs are capability/economics keyed, not intent-semantic keyed.

## 5) Core Technical Decisions

1. Ruthless module replacement
- retire `core/src/mind/*`.
- introduce `core/src/cortex/*`, `core/src/non_cortex/*`, `core/src/spine/*`.
- expose new public surfaces from `core/src/lib.rs`.

2. API break is accepted
- no compatibility shim for `beluna::mind`.
- tests/contracts/docs migrate directly to new canonical names.

3. Attempt vs effect separation becomes primary domain model
- cortex emits `IntentAttempt`.
- non-cortex emits `EffectuationOutcome`:
  - `allowed`
  - `denied_hard`
  - `degraded`
  - `charged_only` (accepted but costly)
- every outcome includes mechanical rationale metadata (constraint code, budget delta), not moral text.

4. Continuity relocation
- continuity state leaves cortex and lives in non-cortex kernel state.
- cortex can be replaced/restarted without losing survival/accounting continuity.

5. Global survival budget ledger
- create one non-cortex global ledger.
- all admitted actions produce ledger impact entries.
- integrate AI Gateway as an initial debit source using approximate costs/tokens/time.
- ledger API designed for later fidelity upgrades without domain break.

6. Spine as contracts-first module
- define spine ports/types and contract docs/tests.
- no concrete runtime/body endpoint integration in this stage.
- provide no-op/test doubles for deterministic tests.

7. Deterministic orchestration preserved
- keep deterministic cycle ordering and stable tie-breaks.
- ensure same starting state + same inputs => same outputs.

8. Docs/contracts migration to canonical terminology
- replace Mind feature/module/contract ownership with Cortex/Non-cortex/Spine artifacts.
- remove terminology drift with product overview and glossary.

## 6) Dependency Direction Rules

1. `cortex` depends on:
- local cortex domain/policy modules,
- shared primitive types,
- non-cortex and spine ports/contracts only (not concrete runtime/body).

2. `non_cortex` depends on:
- local mechanism/ledger/continuity modules,
- spine ports/contracts only for dispatch path.

3. `spine` depends on:
- only shared types and local contract definitions.

4. Prohibited:
- cortex/non-cortex/spine direct dependency on `server.rs`, `protocol.rs`, or concrete body runtime adapters in this phase.

## 7) Migration Strategy (High-Level)

1. Introduce new modules + types/contracts first.
2. Port deterministic behaviors from `mind` into their new ownership:
- deliberative logic -> cortex,
- invariant/mechanical gating/continuity -> non-cortex,
- execution boundary types -> spine.
3. Replace old tests/contracts with new canonical suites.
4. Remove `mind` exports and stale docs once new suites are green.

## 8) Risks and Mitigations

1. Risk: accidental semantic filtering in non-cortex.
- Mitigation: enforce "non-interpretation rule" in contracts and tests.

2. Risk: budget ledger becomes AI-only budget mirror.
- Mitigation: design ledger as global action economics substrate with pluggable debit sources.

3. Risk: continuity accidentally remains in cortex state.
- Mitigation: isolate continuity kernel in non-cortex and test cortex restart/replace scenarios.

4. Risk: spine ambiguity causes over-implementation.
- Mitigation: strict scope control to interface/contracts only in this stage.

5. Risk: large rename causes doc/test mismatch.
- Mitigation: contract-first migration and canonical term sweep.

## 9) Deliverables Expected from L2

L2 should define:
1. exact file/module map for `cortex`, `non_cortex`, `spine`.
2. port interfaces and ownership boundaries.
3. core data structures:
- `IntentAttempt`,
- `EffectuationOutcome`,
- continuity kernel state,
- global survival ledger records.
4. non-cortex affordance/economics algorithms (hard/economic/soft handling).
5. deterministic orchestration algorithm across cortex -> non-cortex -> spine feedback.
6. AI Gateway debit adapter shape into global ledger (approximate v1).
7. contract/test matrix for:
- non-interpretation invariants,
- budget survival mechanics,
- deterministic behavior,
- cortex replaceability continuity.

## 10) L1 Exit Criteria

L1 is complete when accepted:
1. canonical split (`cortex`, `non_cortex`, `spine`) and no compatibility requirement.
2. non-cortex non-interpretation rule as architectural invariant.
3. global survival ledger direction with approximate AI Gateway debits.
4. spine contracts-only scope for current phase.
5. deterministic, mechanism-first enforcement strategy.

Status: `READY_FOR_L2_APPROVAL`
