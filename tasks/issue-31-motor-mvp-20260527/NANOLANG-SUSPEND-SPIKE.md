# Nanolang Suspend / Resume Spike

> Last Updated: 2026-05-30
> Status: exploratory evidence

## Question

Can Nanolang support the desired Motor routine shape?

```text
emit_act -> await_sense -> resume on Afferent Sense -> emit_sense
```

This is stricter than the first embedding spike. The first spike proved:

```text
source string -> parse -> typecheck -> shadow test -> call function
```

This second spike tests whether Nanolang can actually act as a Sense-driven
continuation runtime for Motor.

## Local Spike Artifacts

All spike artifacts are outside the Beluna repo:

- `/tmp/beluna-nanolang-spike/motor_suspend_spike.c`
- `/tmp/beluna-nanolang-spike/motor_event_step_spike.c`
- `/tmp/beluna-nanolang-spike/nanolang`

## Evidence 1: Ordinary Async / Await Works

Command:

```bash
./bin/nanoc tests/test_async.nano -o /tmp/beluna-nanolang-spike/test_async
/tmp/beluna-nanolang-spike/test_async
```

Observed:

```text
async/await tests PASSED
```

Interpretation:

- Nanolang parses and runs basic `async fn` / `await`.
- This only proves transparent / local async behavior.
- It does not prove external host-driven suspension.

## Evidence 2: Motor-Like `await_sense` Does Not Suspend

The tested Nanolang source shape:

```nano
extern fn motor_emit_act(descriptor: string) -> int
extern fn motor_await_sense(act_id: int) -> int
extern fn motor_emit_sense(value: int) -> int

async fn routine() -> int {
    let act_id: int = (motor_emit_act "spine.write-file")
    let sense_value: int = await (motor_await_sense act_id)
    let emitted: int = (motor_emit_sense sense_value)
    return emitted
}
```

Command:

```bash
cc ... /tmp/beluna-nanolang-spike/motor_suspend_spike.c ... \
  -o /tmp/beluna-nanolang-spike/motor_suspend_spike
/tmp/beluna-nanolang-spike/motor_suspend_spike
```

Observed:

```text
case1: call routine before sense exists
host: emit_act descriptor=spine.write-file act_id=1001
host: await_sense act_id=1001 sense_available=false
host: emit_sense value=-1
case1: result_type=0 result=-1 await_calls=1 emitted_sense=-1 pending=0

case2: inject sense and call routine again
host: emit_act descriptor=spine.write-file act_id=1001
host: await_sense act_id=1001 sense_available=true
host: emit_sense value=42
case2: result_type=0 result=42 await_calls=2 emitted_sense=42 pending=0
```

Interpretation:

- `await (motor_await_sense act_id)` evaluates the host function synchronously.
- When the host has no Sense, the routine does not suspend.
- The routine continues and emits `motor_emit_sense(-1)`.
- Scheduler pending count is `0`.
- There is no Motor-visible suspended invocation frame.

This fails the desired coroutine-shaped Motor routine semantics.

## Evidence 3: Source-Level Coroutine Scheduler Surface Is Not Ready

Command:

```bash
./bin/nanoc tests/test_coroutine.nano -o /tmp/beluna-nanolang-spike/test_coroutine
```

Observed:

```text
I cannot find a function named `coro_yield`.
I cannot find a function named `coro_spawn`.
I cannot find a function named `scheduler_run`.
I cannot find a function named `coro_result`.
I cannot find a function named `scheduler_step`.
Type checking failed
```

Interpretation:

- The repo contains coroutine runtime C functions.
- The current language/typechecker surface does not expose them as stable
  Nanolang source-level builtins.
- Even if Motor wanted to manage Nanolang coroutine handles directly, that path
  is not currently usable without Nanolang runtime/compiler work.

## Evidence 4: Nanolang Event-Step Routine Works

The tested fallback shape:

```text
routine_step(event_kind, state, sense_value) -> next_state
```

The tested Nanolang source shape:

```nano
extern fn motor_emit_act(descriptor: string) -> int
extern fn motor_emit_sense(value: int) -> int

fn routine_step(event_kind: int, state: int, sense_value: int) -> int {
    if (== event_kind 0) {
        let act_id: int = (motor_emit_act "spine.write-file")
        return 1
    }

    if (and (== event_kind 1) (== state 1)) {
        let emitted: int = (motor_emit_sense sense_value)
        return 2
    }

    return state
}
```

Command:

```bash
cc ... /tmp/beluna-nanolang-spike/motor_event_step_spike.c ... \
  -o /tmp/beluna-nanolang-spike/motor_event_step_spike
/tmp/beluna-nanolang-spike/motor_event_step_spike
```

Observed:

```text
host: emit_act descriptor=spine.write-file count=1
start: state=1 emitted_act_count=1 emitted_sense=0
host: emit_sense value=42
sense: state=2 emitted_act_count=1 emitted_sense=42
```

Interpretation:

- Nanolang can be embedded for typed event-step routines.
- Motor can own invocation state and call Nanolang repeatedly.
- This shape works, but it gives up the main Nanolang advantage over Rhai:
  natural procedure-shaped routine authoring.

## Runtime Reading

Relevant local code observations:

- `src/cps_pass.c` says the synchronous interpreter treats `await` as
  transparent unless the inner expression returns a coroutine handle.
- `src/eval.c` implements `AST_AWAIT` by evaluating the inner expression and
  only calling `nano_coro_await_id` when the value is `VAL_COROUTINE`.
- `src/coroutine.c` uses a run-to-completion scheduler. Its `yield` function is
  currently a no-op.
- Async function calls in `src/eval.c` spawn a coroutine and immediately await
  it to completion for ordinary calls.

## Conclusion

Current Nanolang should not be selected for a coroutine-shaped Motor MVP unless
we are willing to modify or wrap Nanolang's runtime much more deeply.

The desired source shape:

```nano
let sense = await (await_sense act_id)
```

does not currently mean:

```text
return control to Motor and resume this routine when a matching Sense arrives
```

It currently means:

```text
call await_sense synchronously; if it returns a normal value, continue now
```

Nanolang remains viable only in two narrower ways:

1. As a typed event-step routine language, where Motor owns state and
   continuation.
2. As a future coroutine-shaped language after a real Nanolang runtime change
   that introduces host-visible suspension tokens / continuation handles.

## Decision Impact

This weakens Nanolang as the Motor MVP DSL choice.

If Motor MVP accepts event-step routines, Rhai is still simpler because it gives
Rust-native embedding and mature host APIs.

If Motor MVP requires procedure-shaped routines, neither Rhai nor current
Nanolang gives the exact semantics for free. We would need either:

- a custom tiny Motor DSL with explicit `await_sense` semantics, or
- a deeper Nanolang fork/wrapper that exposes host-driven continuations.
