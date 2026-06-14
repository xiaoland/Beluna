# Efferent Pathway Middleware Response

> Last Updated: 2026-06-13
> Status: superseded for Motor routine modeling

## Current Position

This note records an earlier proposed Efferent Pathway middleware response.

It is no longer the current Motor routine model.

Current Motor routine execution is:

```text
Sense + activation state -> routine -> activation state + Acts
```

Motor is middleware on both Afferent and Efferent Pathways. Lifecycle Acts still
exist, but they are not a separate `Act -> Vec<Act>` control-plane model.

## Historical Position

The response could be much simpler than a `Continue | Complete` enum.

The core middleware shape should be:

```rust
pub type EfferentPathwayMiddlewareResponse = Vec<Act>;
```

Conceptually:

```text
Act -> Vec<Act>
```

This makes each middleware a transformer over the Neural Signal Act stream.

## Semantics

Given one input Act:

1. Pass through:

```rust
vec![act]
```

2. Transform:

```rust
vec![rewritten_act]
```

3. Expand:

```rust
vec![act_a, act_b, act_c]
```

4. Intercept / terminate:

```rust
vec![]
```

If the response is empty, later pipeline components have no Act to process. That means the current Act was consumed by this middleware.

## Historical Motor Mapping

Motor does not own the Act:

```rust
vec![act]
```

Motor owns a lifecycle control Act and consumes it:

```rust
vec![]
```

Motor owns a lifecycle control Act and must emit follow-up persistence or
descriptor Acts:

```rust
next_step_acts
```

Motor owns a lifecycle control Act and only produces lifecycle Senses:

```rust
vec![]
```

This is enough to model:

```text
Motor -> Continuity -> Spine
```

without adding response `kind`, `operation`, `route`, or `Complete` / `Continue` variants.

The response is only the efferent output of this middleware invocation.

It is not the whole routine lifecycle. Under the corrected reflex model, active
routines primarily run because Motor observes matched Senses on the Afferent
Pathway and emits Acts.

## Error Channel

The simple response does not need to encode failure variants.

The middleware function can still return an outer `Result`:

```rust
pub type EfferentPathwayMiddlewareResult =
    Result<EfferentPathwayMiddlewareResponse, EfferentPathwayMiddlewareError>;
```

This keeps two concepts separate:

- `Vec<Act>`: what should downstream middleware process?
- `Err(...)`: the middleware itself failed and the pathway must produce a terminal failure outcome for the original dispatch.

Domain-level rejection does not need to be a response kind. A middleware can consume the Act with `vec![]`. Generic accepted/rejected/failure payloads belong to the efferent pathway, while routine-specific details can be emitted as Senses and observability.

## Existing Similar Shapes

There are two existing nearby concepts:

1. `DispatchDecision`
   - Location: `core/src/types.rs`
   - Shape: `Continue | Break`
   - Current user: Continuity.
   - Limitation: it is a control decision, not a stream transform.
   - Possible replacement: `Continue` maps to `vec![act]`; `Break` maps to `vec![]`.

2. `ActDispatchResult`
   - Location: `core/src/spine/types.rs`
   - Shape: `Acknowledged | Rejected | Lost`
   - Current users: Spine and the efferent caller response channel.
   - Limitation: terminal outcome only; it should not drive middleware stream shape.

## Caller Result Implication

The existing `dispatch_and_wait` path expects one `ActDispatchResult` for the original envelope.

With `Vec<Act>` middleware response, the pathway must define how to derive that terminal result:

- If an Act reaches Spine, use Spine's terminal result.
- If a middleware consumes the Act with `vec![]` successfully, the efferent pathway owns the terminal accepted/rejected payload for the original dispatch.
- If a middleware returns `Err`, map it to `Lost` or another terminal failure.

For Motor specifically, routine semantic completion should arrive through routine-produced afferent Senses, not through the immediate dispatch result.

## Lineage And Loop Guard

Even if the response is just `Vec<Act>`, the pathway still needs lineage outside the response shape.

Likely envelope or context fields:

- producer component
- parent act instance id
- motor invocation id
- routine id
- relation from original Motor lifecycle Act to Motor-emitted child Acts
- activation id
- triggering Sense id
- routine trigger / invocation correlation

These fields should live in the envelope/context/observability layer, not in `EfferentPathwayMiddlewareResponse`.

## Open Questions

1. What exact accepted/rejected payload does the efferent pathway produce for a consumed Act?
2. Should `DispatchDecision` be retired in favor of `Vec<Act>` for Continuity?
3. Where should lineage live: `Act`, `EfferentActEnvelope`, or middleware context?
