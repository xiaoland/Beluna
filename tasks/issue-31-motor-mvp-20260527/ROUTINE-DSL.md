# Routine DSL Decision Notes

> Last Updated: 2026-06-13

## Current Status

The routine DSL is not finally decided.

Nanolang embedding spike was performed on 2026-05-30. Result: C-level
in-process embedding and a Rust-to-C wrapper both worked for a minimal
in-memory routine. See [DSL-COMPARISON-SPIKE.md](./DSL-COMPARISON-SPIKE.md).

A second Nanolang suspend/resume spike was performed on 2026-05-30. Result:
current Nanolang `await` does not provide Motor-visible suspension for
`await_sense`; Nanolang works as a typed event-step substrate but not as a
coroutine-shaped Motor runtime without deeper runtime work. See
[NANOLANG-SUSPEND-SPIKE.md](./NANOLANG-SUSPEND-SPIKE.md).

This remains a first-order design question, but the routine model changed.

The superseded shape was:

```text
Act -> Vec<Act>
```

The current preferred shape is:

```text
state + Sense -> state + Vec<Act>
```

An active routine lets Motor continuously take over procedural action by
reacting mechanically to matched Afferent Senses, updating explicit activation
state, and emitting Efferent Acts.

## External Status Snapshot

Checked on 2026-05-30:

- Nanolang describes itself as a minimal language designed for machines to write and humans to read. Its README also advertises formal semantics, shadow tests, a VM, algebraic effects, and async / await.
- Mojo is currently around the 1.0 beta transition. Official docs position it as a systems / AI infrastructure language with CPU/GPU performance goals, not as an embedded routine DSL.
- Rhai is a Rust-embedded scripting engine with tight Rust integration.
- `mlua` provides Rust bindings for Lua / Luau with async support.

Sources:

- Nanolang README: `https://github.com/jordanhubbard/nanolang`
- Mojo docs: `https://mojolang.org/docs/`
- Mojo roadmap: `https://mojolang.org/docs/roadmap/`
- Mojo 1.0 beta release: `https://mojolang.static.modular-staging.com/releases/v1.0.0b1/`
- Rhai: `https://rhai.rs/`
- mlua: `https://github.com/mlua-rs/mlua`

## Required Semantics

The DSL must support:

- Cortex-authored routine source.
- function-shaped routine definitions.
- create-time validation.
- lifecycle management through built-in Motor Acts.
- persistence of routine definitions through Continuity.
- a Sense-shaped routine input.
- explicit activation state input and output.
- emitting downstream Acts after Motor.
- producing routine-specific Senses.
- reacting to routine-correlated Afferent Senses.
- non-persisted activation / invocation frames for MVP.

## Stateless Routine Interpretation

"Routine is stateless" should mean:

- routine definition has no mutable state beyond its persisted source / descriptor metadata.
- Continuity persists routine definitions.
- Motor does not persist invocation or activation frames across restarts for MVP
  unless the lifecycle decision itself is intentionally persisted.

It does not necessarily mean:

- no active invocation frame exists while a routine is running.

For sustained procedural takeover, Motor needs explicit ephemeral activation state:

- activation id
- active Sense selectors / correlations
- pending child Act ids
- cancellation / termination flag
- observability lineage

This state is runtime-local and non-persisted in MVP.

## Candidate Semantic Shapes

### 1. Pure Act Transform Function

```text
fn routine(payload) -> Vec<Act>
```

Good:

- very simple.

Problem:

- too weak for sustained procedural takeover.
- cannot naturally wait for Afferent Sense feedback.

Current fit:

- superseded as the primary routine model.

### 2. Sense Handler Function

```text
fn routine(state, sense) -> RoutineReaction
```

Good:

- directly matches the corrected Motor model.
- keeps routine source pure from the DSL perspective.
- lets Motor own activation, matching, correlation, and scheduling.
- supports multi-step behavior without hidden mutable runtime state.
- works with simple embedded interpreters.

Concern:

- Motor must define the state value shape and validation rules.

Current fit:

- strongest MVP fit under the reflex model.

### 3. Event Step Function

```text
fn routine(state, event) -> RoutineStepOutput
```

Where `event` can be:

- matched Sense
- activation signal
- timeout / tick / cancellation signal if later needed

Good:

- stays function-shaped.
- supports repeated calls.
- keeps state outside the routine source if Motor supplies event + invocation metadata.

Concern:

- may require explicit continuation metadata in output.

Current fit:

- useful if Sense-only input becomes too narrow.

### 4. Coroutine / Generator Function

```text
fn routine(payload) {
    emit_act(...)
    let sense = await_sense(...)
    emit_act(...)
    emit_sense(...)
}
```

Good:

- natural for procedure-shaped source.
- can express "do this, wait, then do that" directly.

Concern:

- more runtime complexity than the corrected MVP requires.
- Nanolang's current runtime did not expose Motor-visible suspension in the
  second spike.

Current fit:

- no longer required for MVP if Motor owns the event loop.

## Current Recommendation

Do not choose an external DSL package solely because it is a language.

Recommended semantic target is now:

- Sense handler routine with explicit activation state if the reflex model is accepted.
- event step function if activation, cancellation, or timer events need to be
  first-class.
- coroutine / generator-style routine only if source ergonomics becomes more
  important than runtime simplicity.

Avoid selecting a DSL that only supports pure `Act -> Vec<Act>` transforms.

## DSL Candidate Comparison

### Nanolang

Pros:

- designed for machine-written code.
- unambiguous syntax and test-oriented culture match Cortex-authored routines.
- has async / await and algebraic effects, which are semantically close to routine continuation.
- has a VM path, which may be useful if embedding can be made clean.

Cons:

- external young language/toolchain risk.
- not obviously a Rust library that can be embedded as a small in-process interpreter.
- may still be much larger than the Motor DSL actually needs.

Current read:

- technically embeddable enough to keep as a serious candidate.
- not yet a low-cost default because there is no obvious stable Rust crate or
  narrow embedding API.
- much less compelling for MVP if the routine contract is
  `state + Sense -> state + Vec<Act>`,
  because its coroutine-shaped promise is no longer needed.
- current runtime does not give Motor the desired host-visible `await_sense`
  suspension/resumption semantics.
- viable as a typed event-step substrate, but then Rhai is simpler from a Rust
  integration perspective.
- only strongest for procedure-shaped routines if we are willing to change or
  deeply wrap Nanolang's runtime continuation model.

### Mojo

Pros:

- serious language effort with improving documentation and 1.0 beta momentum.
- strong for performance-oriented systems / AI infrastructure work.

Cons:

- too heavy for Motor routine DSL.
- language is still in beta / transition.
- embedding story is not aligned with "Cortex writes small persisted routines".
- its performance and GPU goals are not the bottleneck in Motor MVP.

Current read:

- reject for MVP routine DSL.

### Rhai / Rust-Embedded Scripting

Pros:

- Rust-native embedding.
- small operational footprint compared with external compilers.
- good fit if Motor uses Sense-handler or event-step functions and Motor owns activation state.

Cons:

- not specifically designed for LLM-authored code.
- coroutine / `await_sense` semantics would likely be a Motor runtime convention rather than a native language feature.

Current read:

- strongest low-risk MVP candidate if we prefer implementation speed over
  language purity.
- strong MVP fit under the corrected explicit-state Sense handler model.
- weaker only if we require procedure-shaped source with native suspension.

### Lua / Luau via `mlua`

Pros:

- mature scripting model.
- async-capable Rust bindings exist.
- coroutine-style programming is culturally familiar in Lua-family runtimes.

Cons:

- more dynamic and less self-describing than a Motor-specific DSL.
- LLM authoring may be less constrained unless we heavily restrict the exposed API.

Current read:

- viable fallback, but less aligned with Beluna's Neural Signal vocabulary than a custom DSL.

### Custom Minimal Motor DSL

Pros:

- can encode Beluna primitives directly: `sense`, `act`, `emit`, `selector`.
- smallest semantic surface.
- easiest to make lifecycle validation, selector validation, and observability first-class.
- avoids external language/runtime churn.

Cons:

- more design and implementation work.
- requires parser, validator, evaluator, examples, and Agent Test coverage.

Current read:

- best conceptual fit if we want Beluna-native routine syntax instead of a
  general embedded scripting language.
- strongest long-term alignment with Motor's role.

## Preliminary Decision

For MVP planning:

1. Reject Mojo for routine DSL.
2. Prefer Rhai as the pragmatic embedded-Rust MVP substrate if the routine
   contract is `state + Sense -> state + Vec<Act>`.
3. Keep Nanolang as a serious but higher-cost candidate because the embedding
   spike proved basic C/Rust host viability.
4. Prefer a custom minimal Motor DSL only if we are willing to implement the
   tiny routine surface Motor actually needs.

The next concrete decision should be:

- Rhai Sense-handler MVP vs custom tiny Motor DSL.
- Nanolang-backed coroutine-shaped MVP should be treated as requiring runtime
  work, not as an off-the-shelf integration.

If the goal is fastest implementation:

- use Rhai or a minimal custom explicit-state Sense-handler interpreter.

If the goal is best product fit:

- design a tiny custom Motor DSL around Sense selectors and Act emission.
- keep coroutine-like semantics only if the reflex handler model proves too weak.
- keep Nanolang only if its C wrapper cost is acceptable for stronger static validation.

## Open Questions

1. Is `fn on_sense(state, sense) -> RoutineReaction` sufficient for MVP?
2. Should routine output include Senses explicitly?
3. What shape should explicit activation state use?
4. Is Rhai acceptable as the MVP routine language under the reflex model?
5. Is Nanolang still justified if coroutine semantics are no longer required?
