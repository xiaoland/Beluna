# Architecture Exploration

> Last Updated: 2026-05-28

## Why This Exists

The previous packet wording used "ordinary Act" and "internal control Act" language.

That was misleading for Beluna Core.

The actual architecture is centered on Neural Signals:

- `Act`
- `Sense`
- `endpoint_id`
- `neural_signal_descriptor_id`
- descriptor catalog in physical state

## Evidence

### Core Types

`core/src/types.rs` defines:

- `NeuralSignalType::{Sense, Act}`
- `NeuralSignalDescriptor`
- `Sense`
- `Act`

`Act` and `Sense` both carry:

- `endpoint_id`
- `neural_signal_descriptor_id`

This means Motor should be modeled by endpoint identity and descriptors, not by inventing a separate "ordinary/internal" Act split.

### Coordination Model

`docs/20-product-tdd/coordination-model.md` says efferent dispatch is an ordered pipeline:

```text
Continuity.on_act -> Spine.on_act_final
```

Issue #31 exploration changes the proposed order to:

```text
Motor -> Continuity -> Spine
```

The change is not "remove pipeline"; it is "make the pipeline middleware-like".

### Spine

`core/src/spine/AGENTS.md` states:

- Spine accepts `Act` dispatches only.
- Routing is a mechanical endpoint lookup by `act.endpoint_id`.
- Dispatch failures are emitted back into afferent pathway as domain senses.

`core/src/spine/runtime.rs` confirms:

- `dispatch_act` resolves dispatch by `act.endpoint_id`.
- failure sense uses endpoint id `core.spine` and descriptor id `dispatch.failed`.

### Body Endpoints

`core/src/body/AGENTS.md` states:

- built-in endpoints run in the same process as Core.
- they attach through Spine Inline Adapter.
- endpoint senses are forwarded to `SenseAfferentPathway` by Spine Inline Adapter.

Motor is not identical to those built-in endpoints because Motor must run before Continuity and Spine in the efferent pipeline.

But Motor should still be endpoint-shaped: an inner component with endpoint-owned Neural Signal descriptors.

### Afferent Pathway

`core/src/stem/afferent_pathway.rs` accepts `Sense` values and forwards them through deferral rules to Cortex.

This supports the model:

- Motor itself need not define generic accepted/rejected senses.
- A routine can produce `Sense` values through the afferent pathway.
- Motor can also observe routine-correlated Senses to continue procedural work.
- Efferent pathway owns generic accepted/rejected dispatch outcome semantics for all middleware participants.

## Corrected Modeling Claims

1. Do not use "ordinary Act" as if there are special non-ordinary Acts.
2. All Acts and Senses are Neural Signals.
3. Motor should be modeled as a core-internal endpoint-shaped middleware component.
4. Motor ownership is expressed through `endpoint_id` and descriptor ids.
5. Routine output can include emitted Acts and routine-produced Senses.
6. Generic dispatch accepted/rejected/failure payload authority belongs to the efferent pathway, not Motor.
7. Motor's role is not limited to one-shot Act expansion; it can continue procedural routines across Efferent and Afferent Pathway turns.
