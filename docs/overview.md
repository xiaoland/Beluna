# Beluna Product Overview

Beluna is a life trying to survive and exploring the environemtns; it's NOT a chatbot either an agent.

## Core Invariants

- Natural language as proctol, from Cortex's cognition to Body Endpoint's description; This is the only way to bridge the huge gap between a silicon-based life and carban-based life.
- Cortex is the mind; Cortex is a stateless cognition engine, it only lives current tick.
- Cortex can intend anything and outputs intentions.
- Continutiy is the soul; is the persisted and the only source of principals.
- Ledger is the metabolism; it signals survial pressure driving the evolution.
- Spine is an ignorance channel connects Mind and Body.
- Body (Endpoint) is the surface Beluna contacts the world.
- Body (Endpoint) has ephemeral state.

## Runtime Topology

Beluna Core canonical top-level components:

- Cortex (cognition)
- Continuity (operational state)
- Admission (effectuation gate)
- Ledger (resource control)
- Spine (execution transport)

Operational flow:

```text
Sense + EnvSnapshot stream -> CortexInbox
CortexReactor (always-on) consumes ReactionInput
Primary IR + sub-compile + deterministic clamp -> IntentAttempt[]
IntentAttempt[] -> Admission
Admission checks + queries Ledger -> Ledger reserves
Admission produces AdmittedAction[] -> Spine
Spine executes -> Feedback (attempt_id-correlated) -> Continuity + Ledger settlement
```

## MVP Status

- Cortex/Continuity/Admission/Ledger/Spine contracts are implemented in `core/src/*`.
- Spine adapter is deterministic noop for current MVP.
- AI Gateway debit observations are approximate and attribution-linked.
