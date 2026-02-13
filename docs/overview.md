# Beluna Product Overview

Beluna is a survival-oriented digital life runtime, not a chatbot.

## Core Invariants

1. Natural language is the protocol across cognition and body affordances.
2. Cortex is the mind and is stateless per reaction tick.
3. Cortex can intend anything and emits non-binding intentions.
4. Continuity is the persisted operational memory and policy anchor.
5. Ledger is metabolism and survival pressure.
6. Spine is the transport-ignorant channel between Mind and Body.
7. Body (Endpoints) is Beluna's interface to the world.

## Runtime Topology

Beluna runtime process:
1. `core` runnable binary (`beluna`) with embedded std body endpoints.
2. Optional external Body Endpoints (for example Apple Universal App) connect over UnixSocket.

Beluna Core top-level components:
1. Cortex (cognition)
2. Continuity (operational state)
3. Admission (effectuation gate)
4. Ledger (resource control)
5. Spine (mechanical execution routing)

Operational flow:

```text
Sense + EnvSnapshot stream -> CortexInbox
CortexReactor consumes ReactionInput
Primary IR + sub-compile + deterministic clamp -> IntentAttempt[]
IntentAttempt[] (Neural Signal queue) -> Admission
Admission + Ledger -> AdmittedActionBatch
AdmittedActionBatch -> Spine (route lookup + endpoint dispatch over body endpoints)
SpineExecutionReport -> Continuity + Ledger settlement
```

## Current MVP Status

1. Cortex/Continuity/Admission/Ledger/Spine contracts are implemented in `core/src/*`.
2. Spine ships async routing kernel + in-memory registry.
3. UnixSocket adapter is the active Spine shell for ingress and body endpoint lifecycle (`body_endpoint_register`/`body_endpoint_invoke`/`body_endpoint_result`).
4. Core embeds Shell and Web standard body endpoints (config-gated).
5. Apple Universal App acts as an independent chat Body Endpoint and self-registers with Spine UnixSocket.
