# Pathway Middleware Contract

> Last Updated: 2026-06-14
> Status: proposed prerequisite

## Core Correction

Do not model `Terminal` as a fixed component role.

In a pathway middleware chain, any middleware can become terminal for a signal it
owns.

Examples:

- `continuity.l1_memory.update` may be accepted by Continuity and never reach
  Spine.
- `motor.routine.activate` may be accepted by Motor and never reach Spine.
- endpoint-directed Acts continue until Spine handles endpoint dispatch.

Therefore the important abstraction is the middleware response contract.

## Middleware Decision

For one input signal, middleware returns one of:

```text
Accepted
Rejected
Continue(original | transformed)
```

Meaning:

- `Accepted`: this middleware handled the signal successfully; downstream
  middleware has nothing to do for this signal.
- `Rejected`: this middleware refuses the signal; downstream middleware has
  nothing to do for this signal, and the pathway should produce a failure
  outcome / feedback.
- `Continue(original)`: pass the original signal downstream.
- `Continue(transformed)`: pass replacement same-type signals downstream.

This replaces the earlier ambiguous `Vec<Signal>` idea.

## Shape

Conceptual Rust shape:

```rust
pub enum PathwayMiddlewareDecision<S> {
    Accepted {
        receipt: PathwayReceipt,
    },
    Rejected {
        reason_code: String,
        message: Option<String>,
    },
    Continue {
        output: ContinueOutput<S>,
    },
}

pub enum ContinueOutput<S> {
    Original,
    Replace(Vec<S>),
}
```

Rules:

- `ContinueOutput::Original` means the input signal continues unchanged.
- `ContinueOutput::Replace(signals)` means the input is transformed before the
  remaining middleware sequence continues.
- The pathway contract does not assign domain meaning to one-to-one,
  one-to-many, many-to-one, or filtering transforms. Those are middleware-local
  behaviors.

## Current Signal vs New Signals

The middleware decision controls the fate of the current signal.

`Continue` is same-pathway and same-signal-type only.

Examples:

- Afferent middleware receives `Sense`, so `Continue(...)` can only continue
  `Sense`.
- Efferent middleware receives `Act`, so `Continue(...)` can only continue
  `Act`.

Therefore `Sense -> Act` is not represented by `Continue`.

Additional cross-pathway emissions should use source ports:

- Afferent middleware may source Efferent Acts.
- Efferent middleware may source Afferent Senses.

These emissions are side effects with their own signal lifecycle. They should
not be hidden inside `Continue`.

## Source Port

A source port is a capability that lets a component introduce a new signal into a
pathway.

Current nearby code shapes:

- `SenseAfferentPathway::send(sense)` introduces Senses into Afferent.
- `ActProducerHandle::enqueue(...)` / `dispatch_and_wait(...)` introduces Acts
  into Efferent.

The proposed model should make this explicit without giving Afferent/Efferent
responsibility for cross-pathway loop prevention.

Conceptual shape:

```text
AfferentSourcePort.emit_sense(sense)
EfferentSourcePort.emit_act(act)
```

The source port API does not accept source metadata. If an implementation needs
runtime attribution, it can bind an internal emitter id to the Tx handle at
construction time. That id is not part of the pathway contract and is not
visible to middleware.

This keeps two concepts separate:

- middleware decision: what happens to the current signal?
- source emission: what new signals are introduced into a pathway?

## Efferent Examples

### Pass endpoint Act to Spine

```text
Motor: Continue(original)
Continuity: Continue(original)
Spine: Accepted
```

### Motor handles routine activation

```text
Motor: Accepted
```

Motor may also source a `motor.routine.activated` Sense into Afferent.

### Continuity handles memory update

```text
Motor: Continue(original)
Continuity: Accepted
```

Continuity may persist a store record and source an update-result Sense if the
descriptor contract requires feedback.

### Rejected Act

```text
Motor: Rejected(reason_code = "unknown_routine")
```

The Efferent Pathway owns conversion of this middleware rejection into terminal
dispatch outcome and/or feedback Sense.

## Afferent Examples

### Pass Sense to Cortex

```text
Motor: Continue(original)
Cortex: Accepted
```

Cortex accepting here means it has admitted the Sense into cognition input, not
that the world event is semantically resolved.

### Motor consumes a reflex Sense

```text
Motor: Accepted
```

Motor may source Acts into Efferent as routine output.

Open policy:

- Should Cortex still receive a summary / consumed marker Sense?
- Or is pathway observability enough?

### Motor transforms a Sense

```text
Motor: Continue(transformed)
Cortex: Accepted
```

The pathway records the middleware transform event, but does not impose a
cross-pathway loop-prevention policy.

## Spine And Cortex Under This Model

Spine is not "the terminal component" globally.

Spine is the middleware that normally terminally accepts or rejects
endpoint-directed Efferent Acts.

Cortex is not "the terminal component" globally.

Cortex is the middleware that normally terminally accepts Afferent Senses into
cognition.

This keeps Cortex and Spine inside the pathway model without erasing their
authority:

- Cortex owns cognition admission and LLM-driven transformation.
- Spine owns endpoint routing and endpoint dispatch terminal outcomes.

## Open Decisions

1. Should `Accepted` include a typed receipt payload, or only observability
   metadata?
2. Should `Rejected` map to existing `ActDispatchResult::Rejected` on Efferent?
3. What is the Afferent equivalent of rejected / accepted terminal outcome?
4. Should middleware responses be sync values, async results, or stream outputs?
