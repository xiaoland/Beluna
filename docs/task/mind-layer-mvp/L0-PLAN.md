# L0 Plan - Mind Layer MVP

- Task Name: `mind-layer-mvp`
- Stage: `L0` (request + context analysis only)
- Date: 2026-02-08

## 1) Problem Deconstruction

Beluna needs a minimum "Mind" layer that is a meta-level controller, not another model or infra subsystem.

Requested core responsibilities:

1. Goal management
- Keep one active goal at a time.
- Support layered goal structure:
  - high-level intent
  - mid-level objectives
  - low-level executable tasks
- Allow multiple goals to exist, but enforce single-active invariant.

2. Evaluation and judgment (normative, not predictive)
- Judge alignment of outputs with active goal.
- Judge subsystem reliability.
- Judge signal faithfulness (input quality/reality match).

3. Delegation to helpers
- Delegate workload to helper components/sub-agents.
- Aggregate helper results.
- Resolve conflicts deterministically.
- Helpers are extensions under Mind control, not independent minds.

4. Self-evolution trigger
- Decide when a component is "good enough" vs needs change.
- Trigger candidate replace/retrain/reconfigure decisions only when evaluation justifies it.
- Cover model, memory structures, and perception pipelines.

## 2) Current Codebase Reality (Beluna)

Observed from `/Users/lanzhijiang/Development/Beluna/src` and docs:

- Runtime currently provides:
  - strict config loading with JSON Schema validation (`src/config.rs`, `beluna.schema.json`)
  - Unix socket NDJSON listener loop and graceful exit (`src/server.rs`)
  - strict protocol parsing with `deny_unknown_fields` (`src/protocol.rs`)
- AI Gateway MVP exists as an internal module boundary (`src/ai_gateway/*`) and is not integrated into socket protocol yet.
- There is no existing `mind` module, no goal state model, and no helper orchestration runtime boundary.
- Test style is BDD-flavored naming (`given_when_then`) with deterministic unit/e2e coverage (mainly AI Gateway).
- Product docs already define Mind as Layer 3 conceptually (`docs/product/overview.md`) but not as executable contracts yet.

Implication: Mind should be introduced as a strict internal module with deterministic contracts and tests first, without breaking current socket behavior.

## 3) Constraints and Architectural Trade-offs Identified

1. Single active goal vs multi-goal backlog
- Need both: a queue/backlog of goals and a hard invariant that exactly zero or one goal is active.
- Trade-off: simple state machine now vs richer scheduling policy later.

2. Layered goals vs loose task lists
- If goal hierarchy is untyped, evaluation becomes ambiguous.
- Need explicit parent-child linkage and goal level enum for deterministic judgment.

3. Normative evaluation vs heuristic scoring sprawl
- Mind must output explicit judgments with reason/evidence instead of opaque scores only.
- Trade-off: more verbose result model, but auditable evolution decisions.

4. Delegation power vs helper autonomy
- Helpers should not mutate global state or goal state directly.
- Need request/response boundary where Mind remains the single authority.

5. Conflict resolution determinism
- When helper outputs disagree, tie-break policy must be explicit and stable.
- Trade-off: deterministic policy may be conservative but avoids hidden behavior.

6. Evolution trigger safety
- "Evolve" should be decision output first, not direct mutation side-effect in MVP.
- Trade-off: proposal-only MVP is safer and testable, but does not auto-remediate.

7. Integration surface scope
- Option A: integrate immediately with socket protocol.
- Option B: build pure internal Mind core first, protocol integration later.
- Given current runtime maturity, Option B lowers risk and preserves strictness.

## 4) External Source Findings (Firecrawl research)

Research was gathered with Firecrawl and used as architecture reality checks, not as code truth.

1. Hierarchical planning foundations (HTN)
- Source: [An Overview of Hierarchical Task Network Planning](https://arxiv.org/abs/1403.7426)
- Relevance: supports explicit hierarchical task decomposition and rich domain constraints, matching layered goals.

2. Reasoning-action loop with environment feedback
- Source: [ReAct: Synergizing Reasoning and Acting in Language Models](https://arxiv.org/abs/2210.03629)
- Relevance: supports a loop where decisions are updated from interaction feedback, aligning with Mind evaluation/delegation cycle.

3. Reflection-based improvement without weight updates
- Source: [Reflexion: Language Agents with Verbal Reinforcement Learning](https://arxiv.org/abs/2303.11366)
- Relevance: supports explicit evaluation-memory-trigger pattern for controlled improvement decisions.

4. Agent evaluation dimensions and gaps
- Source: [Survey on Evaluation of LLM-based Agents](https://arxiv.org/abs/2503.16416)
- Relevance: confirms planning/tool-use/self-reflection/memory as core evaluation dimensions and highlights robustness/safety/cost gaps.

5. Autonomic meta-control architecture
- Source: [An architectural approach to autonomic computing](https://research.ibm.com/publications/an-architectural-approach-to-autonomic-computing)
- Relevance: supports interface-driven meta-control with explicit behavior requirements between components.

Key implication for Beluna Mind MVP:
- Build Mind as a strict controller with explicit interfaces, hierarchical goal model, auditable evaluations, and proposal-based evolution triggers.

## 5) Initial Scope Boundary (L0 Recommendation)

Recommend explicitly scoping Mind MVP to:

- Internal module boundary only (`src/mind/*`), no socket protocol changes in this task.
- In-memory goal state with one-active-goal invariant.
- Goal hierarchy model (high/mid/low) and deterministic state transitions.
- Evaluation engine that emits structured judgments with reasons and confidence/evidence fields.
- Helper orchestration boundary via trait interfaces; include aggregation and deterministic conflict resolver.
- Evolution decision engine that emits `NoChange` or `ChangeProposal` (replace/reconfigure/retrain), but does not execute changes.
- BDD contracts and tests for invariants and decision rules.

Out of scope for MVP:

- Auto-application of evolution actions.
- Persistent memory store or long-term model training pipeline.
- GUI or protocol-level user interaction flow.

## 6) Open Questions Requiring User Decision

1. Integration timing
- Should this task keep Mind internal-only, or also expose Mind operations through Unix socket protocol now?

2. Evolution execution mode
- Should MVP remain proposal-only (recommended), or include actual runtime reconfiguration side-effects?

3. Helper execution backend
- Should helpers be mocked/trait-based only for MVP (recommended), or include real process spawning in this task?

4. Goal preemption policy
- When a higher-priority goal arrives, should active goal be paused, cancelled, or remain until completion?

## 7) Proposed Working Assumptions (if you do not override)

- Mind MVP is internal-only module and test surface.
- Evolution is decision/proposal only; no automatic component mutation.
- Helper delegation uses trait-based adapters and deterministic test doubles.
- Goal preemption policy for MVP: explicit manual switch only (no automatic preemption).
- Keep protocol and server behavior unchanged.

## 8) Exit Criteria for L0

This stage is complete when:

- request is deconstructed,
- current code constraints are documented,
- architectural trade-offs are explicit,
- external references are captured,
- open decisions and assumptions are listed for stage gating.

Status: `READY_FOR_L1_APPROVAL`
