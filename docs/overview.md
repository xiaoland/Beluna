# Beluna Product Overview

Beluna is a long-running, goal-driven, embodied agent system whose deliberative core (cortex) operates inside mechanistically enforced constraints and economic limits.

Key properties:

- Goal-oriented, not prompt-oriented
- Continuous, not request/response
- Embodied, can change external state
- Constrained, not sovereign
- Economically bounded, not infinitely powered
- Replaceable cognition, stable continuity

Beluna is a closed-loop control system, it's fundamentally:

```text
Perceive → Deliberate → Act → Experience consequence → Adapt
```

Beluna is an environment-shaped agent, Beluna’s behavior emerges from Cortex intentions, Runtime physics (constraints), Budget economics, External world feedback; And so Beluna is shaped by resistance.

Beluna is not a moral system, It does not obey because it is “told to obey.”.
It acts within affordances, adapts to constraint feedback, optimizes under resource survival pressure.

Beluna is NOT a chatbot, free sovereign AI, memory system, rule engine, a centralized authority hierarchy

## Runtime Topology

Beluna Core canonical top-level components:

- Cortex (cognition)
- Continuity (operational state)
- Admission (effectuation gate)
- Ledger (resource control)
- Spine (execution transport)

Operational flow:

```text
Sense -> Continuity
Continuity builds SituationView -> Cortex
Cortex emits IntentAttempt[] -> Admission
Admission checks + queries Ledger -> Ledger reserves
Admission produces AdmittedAction[] -> Spine
Spine executes -> Feedback -> Continuity + Ledger settlement
```

## Core Invariants

- Cortex may intend anything.
- Cortex must manage goals to form a self.
- The cortex operates within encoded priors (inductive biases), not as a blank learner.
- Only effectuation is gated; intention remains free.
- Constraints are runtime affordances and economics, not narrated memory.
- Continuity and safety are mechanism-based, not dependent on cortex internals.
- Budget is survival: unsustainable behavior loses capability.

## MVP Status

- Cortex/Continuity/Admission/Ledger/Spine contracts are implemented in `core/src/*`.
- Spine adapter is deterministic noop for current MVP.
- AI Gateway debit observations are approximate and attribution-linked.
