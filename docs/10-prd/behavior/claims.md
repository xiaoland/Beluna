# Product Claims

This file defines Beluna's primary product claims and embeds evaluation dimensions directly in each claim.

Claim evaluation is external-first: benchmark literature defines dimension framing, then Beluna runtime evidence calibrates practical interpretation.

## Claim 1 - Coherent Continuity Across Cycles

Beluna preserves cognition continuity across runtime cycles so operators can maintain long-horizon intent without manual state reconstruction.

Evaluation dimensions:

- Memory continuity over multi-turn operation.
- Reflection/adjustment quality after failures.
- Robustness under restart and recovery paths.

Evidence expectation:

- Recovery runs restore usable cognition state without operator-side state replay.
- Post-incident behavior stays aligned with pre-incident intent after bounded recovery.
- Regression tests and incident reviews show no repeated continuity-loss pattern.

Source rationale:

- External: [Survey on Evaluation of LLM-based Agents](https://arxiv.org/abs/2503.16416) (memory, self-reflection, robustness dimensions).
- Repo calibration: continuity persistence/guardrails and recovery contracts in current TDD/deployment docs.

Realization pointers:

- `docs/20-product-tdd/data-and-state.md`
- `docs/20-product-tdd/coordination.md`
- `docs/30-unit-tdd/core/interfaces.md`
- `docs/40-deployment/rollout-and-recovery.md`

## Claim 2 - Reliable World Interaction Through Endpoints

Beluna interacts with world affordances through endpoints with explicit outcomes and stable integration expectations.

Evaluation dimensions:

- Tool/use-action reliability and explicit outcome semantics.
- External integration compatibility and contract clarity.
- End-to-end action correctness from operator request to endpoint effect.

Evidence expectation:

- Endpoint integrations show deterministic terminal outcomes per action request.
- Protocol compatibility tests remain stable across CLI and Apple endpoint surfaces.
- Incident analysis can attribute failures to integration/transport/runtime causes without ambiguity.

Source rationale:

- External: [ReAct](https://arxiv.org/abs/2210.03629) and the agent-evaluation survey (tool-use and interactive reliability dimensions).
- Repo calibration: endpoint protocol contracts, dispatch outcome semantics, and adapter lifecycle rules.

Realization pointers:

- `docs/20-product-tdd/coordination.md`
- `docs/20-product-tdd/unit-boundaries.md`
- `docs/30-unit-tdd/cli/interfaces.md`
- `docs/30-unit-tdd/apple-universal/interfaces.md`

## Claim 3 - Operationally Trustworthy Runtime

Beluna remains operationally trustworthy through explicit observability, bounded recovery behavior, and controlled failure handling.

Evaluation dimensions:

- Operational observability quality.
- Safety/reliability under degraded conditions.
- Cost/risk awareness for sustained service operation.

Evidence expectation:

- Operators can diagnose service regressions using logs/metrics/traces without code archaeology.
- Shutdown/recovery paths remain bounded and reproducible in operational checks.
- Reliability/operational constraints are explicitly documented and testable at system boundaries.

Source rationale:

- External: [Survey on Evaluation of LLM-based Agents](https://arxiv.org/abs/2503.16416) (robustness, safety, cost gaps) and [An architectural approach to autonomic computing](https://research.ibm.com/publications/an-architectural-approach-to-autonomic-computing) (operational control principles).
- Repo calibration: current observability ownership model, runtime shutdown contracts, and deployment runbooks.

Realization pointers:

- `docs/20-product-tdd/operational-constraints.md`
- `docs/30-unit-tdd/core/operations.md`
- `docs/40-deployment/observability.md`
- `docs/40-deployment/environments.md`
