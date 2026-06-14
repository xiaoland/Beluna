# Routine DSL Boundaries

> Last Updated: 2026-06-13
> Status: exploratory analysis

## Rhai Boundary

Rhai's boundary is not "can it run scripts inside Rust". It can, and that part
is mature.

Rhai's real boundary for Motor is semantic:

- It is best at synchronous host-orchestrated scripting.
- It is best when Motor owns the routine lifecycle and calls a script function
  for one bounded step.
- It is weaker when the routine source itself should look like a long-running
  procedure that naturally suspends on afferent Senses and resumes later.

Under the corrected reflex model, this boundary is less damaging because the
primary routine contract can be a bounded explicit-state Sense handler.

## Rhai Strengths

Rhai gives Motor a low-risk MVP substrate:

- Rust-native embedding.
- `Engine` can compile routine source to an `AST`.
- Rust can call script-defined functions via `call_fn`.
- Motor can register native Rust functions such as `emit_act`,
  `emit_sense`, `parse_json`, or `validate_payload`.
- Motor can expose selected Rust types or keep the boundary as object maps and
  arrays.
- Engine limits exist for operations, call depth, expression depth, array size,
  map size, string size, variables, and modules.

For Motor MVP, the clean Rhai shape is now:

```text
on_sense(state, sense) -> { state, acts, senses, status }
```

This keeps the runtime simple:

- Motor stores explicit ephemeral activation state.
- Rhai routine source computes a bounded reaction to one Sense and returns next state.
- Later Afferent Senses become later calls into the same active routine.

## Rhai Upper Limit

Rhai becomes less attractive when Motor needs:

- native `await_sense` syntax
- coroutine-shaped routine source
- source-level typed Act / Sense schemas
- mandatory shadow-test-like validation culture
- strong static rejection before execution
- language-level effect boundaries
- a routine that reads as a procedure instead of a reducer / state machine

Rhai can still simulate these, but they become Motor conventions:

- `await_sense` becomes "return Acts and next state now; later Senses call `on_sense` again".
- type checking becomes host-side schema validation.
- tests become host-invoked `test_*` functions.
- routine suspension becomes explicit Motor-owned activation state, not language-owned continuation.

This is acceptable for MVP if we accept routine source as a reflex handler. It
is a product-fit compromise only if Cortex is expected to author readable
procedure-shaped routines.

## Nanolang C Runtime Wrapper Impact

The Nanolang spike showed that integration is technically viable:

- C-level in-process embedding worked.
- Rust-to-C wrapper worked.
- compiler, interpreter, and VM paths built and ran locally.

The second Nanolang spike showed a stricter limit:

- `await (motor_await_sense act_id)` does not suspend the routine when the host
  has no Sense.
- the routine continues synchronously with the host return value.
- scheduler pending count remains `0`.
- Nanolang event-step routines work, but that shape gives up native procedural
  takeover.

But integrating Nanolang into Beluna is not "add a Rust crate". Beluna would own
a wrapper boundary around a C runtime.

## What The Wrapper Must Own

A Beluna-owned wrapper would need to provide:

- build integration for Nanolang C sources or a vendored/static library
- Rust FFI bindings over a tiny stable API
- lifecycle management for `Environment`, `ASTNode`, tokens, and runtime cleanup
- source create pipeline: parse, typecheck, run shadow tests, materialize
  callable routine
- value conversion between Rust `Act` / `Sense` payloads and Nanolang `Value`
- error conversion into Motor create failure / runtime failure Senses
- observability hooks around parse, validation, call, emitted Acts, produced
  Senses, and failures
- concurrency policy, because the C runtime was not proven thread-safe
- panic/crash containment policy if the C side aborts or corrupts process state
- version pinning and update policy for the vendored Nanolang code

## Engineering Consequences

### Build And Packaging

Beluna core is Rust. Nanolang is a C project with a large object set.

Consequences:

- `build.rs` complexity increases.
- CI must have C toolchain and required native libraries.
- macOS/Linux build differences become part of core maintenance.
- cross-compilation gets harder.
- binary size and build time likely increase.

### API Stability

The spike used internal-looking C APIs:

```c
tokenize
parse_program
type_check_module
run_shadow_tests
run_program
call_function
```

Consequences:

- Beluna should not expose those directly across the Rust codebase.
- We would need a narrow `motor_routine_runtime` wrapper API.
- Nanolang upgrades may break the wrapper even if language behavior remains
  compatible.

### Runtime Ownership

The spike host had to provide globals such as:

```c
g_argc
g_argv
get_project_root
```

Consequences:

- the embedded runtime expects process-level context.
- multiple independent Motor runtimes may be awkward.
- tests must prove repeated create/execution/cleanup does not leak or
  cross-contaminate state.

### Safety And Failure Isolation

Security boundary is not the top priority for this issue, but process integrity
still matters.

Consequences:

- in-process C embedding can crash the whole Beluna core.
- a subprocess/VM daemon integration would isolate failures better but adds IPC
  and state synchronization.
- the wrapper must define how Motor reports C-side failures as routine failures.

### Routine Semantics

The simple spike proved:

```text
source string -> parse -> typecheck -> shadow test -> call function -> result
```

The suspend/resume spike then disproved the desired off-the-shelf coroutine
shape. Nanolang did not provide:

- clean routine suspension across afferent Senses
- resuming a Nanolang async frame from Motor later
- stable mapping from `await_sense` to Beluna's Afferent Pathway
- typed Act/Sense value ergonomics at routine authoring level

So current Nanolang is not enough for coroutine-shaped Motor routines without
runtime changes.

Under the corrected reflex model, this is no longer fatal. Nanolang can still be
used as a typed Sense-handler substrate, but that removes the main reason to pay
for its C wrapper in MVP.

## Decision Pressure

Use Rhai when the MVP priority is:

- low integration risk
- explicit-state Sense-handler or event-step routine model
- fast Agent Test feedback
- Rust-native observability

Use Nanolang only if the MVP priority is:

- Cortex-authored routines that look procedural
- create-time static validation
- shadow-test culture
- willingness to implement or own deeper runtime continuation semantics

The practical recommendation remains:

- Rhai is the fastest credible MVP substrate under
  `state + Sense -> state + Vec<Act>`.
- Nanolang is viable as a typed Sense-handler / event-step substrate, but Rhai is simpler for
  that same shape.
- If first-class procedural routine source is core to Motor MVP, prefer a custom
  tiny Motor DSL or budget a deeper Nanolang runtime fork/wrapper.

## Sources

- Rhai Rust functions: `https://rhai.rs/book/rust/functions.html`
- Rhai max operations: `https://rhai.rs/book/safety/max-operations.html`
- Rhai max call stack: `https://rhai.rs/book/safety/max-call-stack.html`
- Rhai expression depth: `https://rhai.rs/book/safety/max-stmt-depth.html`
- Rhai array size: `https://rhai.rs/book/safety/max-array-size.html`
- Rhai map size: `https://rhai.rs/book/safety/max-map-size.html`
- Nanolang repository: `https://github.com/jordanhubbard/nanolang`
