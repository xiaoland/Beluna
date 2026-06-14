# Routine DSL Comparison and Nanolang Embedding Spike

> Last Updated: 2026-06-13
> Status: exploratory evidence

## Scope

This note compares Nanolang and Rhai as Motor routine DSL candidates and records
the local Nanolang embedding spike.

Important update:

- These examples were written before the 2026-06-13 reflex-model correction.
- They are still useful for DSL evidence.
- They should not be read as the current Motor routine contract.
- The current preferred routine shape is
  `state + Sense -> state + Vec<Act>`, not
  coroutine-shaped `emit_act -> await_sense`.

The examples below use Beluna-flavored host APIs such as `emit_act`,
`await_sense`, `json1`, `json2`, and `Act`. Those APIs do not exist yet. The
point is to compare how each language would express the same Motor routine
contract if Motor exposed those host functions and types.

## Direct Example 1: Build Downstream Acts

### Nanolang

```nano
struct Act {
    endpoint_id: string,
    neural_signal_descriptor_id: string,
    payload_json: string
}

fn act(endpoint_id: string, descriptor_id: string, payload_json: string) -> Act {
    return Act {
        endpoint_id: endpoint_id,
        neural_signal_descriptor_id: descriptor_id,
        payload_json: payload_json
    }
}

shadow act {
    assert (== (act "spine" "spine.write-file" "{}").endpoint_id "spine")
}

fn write_markdown(path: string, body: string) -> List<Act> {
    let acts: List<Act> = (List_Act_new)

    let write_payload: string = (json2 "path" path "body" body)
    let validate_payload: string = (json1 "path" path)

    (List_Act_push acts (act "spine" "spine.write-file" write_payload))
    (List_Act_push acts (act "spine" "spine.validate-artifact" validate_payload))

    return acts
}

shadow write_markdown {
    assert (== (List_Act_length (write_markdown "deck.md" "# Title")) 2)
}
```

Read:

- Better at making the routine definition self-describing.
- Static function signatures and `shadow` tests align with registration-time validation.
- Verbose for JSON-like payload construction unless Motor provides typed host
  constructors.

### Rhai

```rhai
fn write_markdown(path, body) {
    [
        #{
            endpoint_id: "spine",
            neural_signal_descriptor_id: "spine.write-file",
            payload_json: `{"path":"${path}","body":"${body}"}`
        },
        #{
            endpoint_id: "spine",
            neural_signal_descriptor_id: "spine.validate-artifact",
            payload_json: `{"path":"${path}"}`
        }
    ]
}
```

Read:

- Much lighter for ad hoc arrays and object maps.
- Return shape is runtime-checked unless Motor adds a validator.
- Tests are a host convention, not a built-in language habit.

## Direct Example 2: Sustained Procedural Takeover

### Nanolang

```nano
struct Sense {
    neural_signal_descriptor_id: string,
    payload_json: string
}

async fn revise_markdown(path: string, body: string) -> void {
    let write_id: string =
        (emit_act "spine" "spine.write-file" (+ path body))

    let written: Sense =
        await (await_sense write_id "spine.write-file.completed")

    let validate_id: string =
        (emit_act "spine" "spine.validate-artifact" path)

    let checked: Sense =
        await (await_sense validate_id "spine.artifact-validation.completed")

    (emit_sense "motor.revise-markdown.completed" checked.payload_json)
}

shadow revise_markdown {
    assert true
}
```

Read:

- Native `async fn` / `await` syntax matches the desired procedural shape.
- Motor could map `await_sense` to an afferent continuation boundary.
- The real embedding question is whether Motor can suspend/resume Nanolang
  frames cleanly enough without turning the C runtime into a large subsystem.

### Rhai

```rhai
fn revise_markdown_step(event) {
    if event.kind == "start" {
        return #{
            state: #{ phase: "waiting-write" },
            acts: [
                #{
                    endpoint_id: "spine",
                    neural_signal_descriptor_id: "spine.write-file",
                    payload_json: event.payload_json
                }
            ],
            senses: []
        };
    }

    if event.kind == "sense" && event.state.phase == "waiting-write" {
        return #{
            state: #{ phase: "waiting-validation" },
            acts: [
                #{
                    endpoint_id: "spine",
                    neural_signal_descriptor_id: "spine.validate-artifact",
                    payload_json: event.path
                }
            ],
            senses: []
        };
    }

    return #{ state: event.state, acts: [], senses: [] };
}
```

Read:

- Rhai has a clean Rust embedding story, but sustained takeover is naturally an
  event-step convention, not a native coroutine expression.
- Motor would own continuation state explicitly.
- This is easier to implement for MVP, but less close to the mental model
  Cortex should write when it wants a procedure.

## Direct Example 3: Registration Validation

### Nanolang

```nano
fn descriptor_id() -> string {
    return "motor.revise-markdown"
}

shadow descriptor_id {
    assert (== (descriptor_id) "motor.revise-markdown")
}
```

Motor can reject registration when:

- parsing fails
- type checking fails
- required exported routine function is absent
- `shadow` tests fail
- descriptor metadata does not match the Act Neural Signal being registered

### Rhai

```rhai
fn descriptor_id() {
    "motor.revise-markdown"
}

fn test_descriptor_id() {
    if descriptor_id() != "motor.revise-markdown" {
        throw "descriptor mismatch";
    }
}
```

Motor can reject registration when:

- compilation fails
- required functions are absent
- host-invoked test functions fail
- returned values do not validate against Motor's routine schema

## Direct Example 4: Host Embedding Shape

### Nanolang C API Shape

The local spike used the repository's C-level API directly:

```c
Token *tokens = tokenize(source, &token_count);
ASTNode *program = parse_program(tokens, token_count);
Environment *env = create_environment();

typecheck_set_current_file("<embedded>");
type_check_module(program, env);
run_shadow_tests(program, env, false);

cps_pass(program);
nano_scheduler_init();
run_program(program, env);

Value arg = create_int(41);
Value result = call_function("inc", &arg, 1, env);
```

Read:

- A C-level in-process embedding exists in practice.
- A Rust host can call it through a C wrapper.
- This is not yet a stable Rust package boundary.

### Rhai Rust API Shape

```rust
use rhai::{Engine, Scope};

let mut engine = Engine::new();
engine.set_max_operations(10_000);
engine.set_max_call_levels(32);

engine.register_fn("emit_act", emit_act);
engine.register_fn("emit_sense", emit_sense);

let ast = engine.compile(routine_source)?;
let mut scope = Scope::new();

let output = engine.call_fn::<rhai::Array>(
    &mut scope,
    &ast,
    "write_markdown",
    (path, body),
)?;
```

Read:

- Rhai is already shaped as a Rust-embedded scripting engine.
- Host functions, compiled AST reuse, and resource limits are first-class APIs.
- Motor's routine semantics must be imposed by our host contract.

## Nanolang Embedding Spike

Workspace:

- clone: `/tmp/beluna-nanolang-spike/nanolang`
- C embedding host: `/tmp/beluna-nanolang-spike/embed_host.c`
- Rust embedding host: `/tmp/beluna-nanolang-spike/rust_embed`

Commands and observed results:

```bash
git clone --depth 1 https://github.com/jordanhubbard/nanolang.git /tmp/beluna-nanolang-spike/nanolang
make build
```

Result:

- passed
- built `bin/nanoc`, `bin/nanoc_c`, and `bin/nano`
- completed the repository's three-stage bootstrap validation

```bash
./bin/nanoc examples/language/nl_hello.nano -o /tmp/beluna-nanolang-spike/nl_hello
/tmp/beluna-nanolang-spike/nl_hello
./bin/nano examples/language/nl_hello.nano
```

Result:

- both paths printed `Hello from NanoLang!`

```bash
make vm
./bin/nano_virt examples/language/nl_hello.nano --run
./bin/nano_virt examples/language/nl_hello.nano --emit-nvm -o /tmp/beluna-nanolang-spike/nl_hello.nvm
./bin/nano_vm /tmp/beluna-nanolang-spike/nl_hello.nvm
```

Result:

- VM backend built
- VM direct run and `.nvm` execution both printed `Hello from NanoLang!`

```bash
cc ... /tmp/beluna-nanolang-spike/embed_host.c ... -o /tmp/beluna-nanolang-spike/embed_host
/tmp/beluna-nanolang-spike/embed_host
```

Result:

- printed `embedded_result=42`
- proved C-level in-process embedding from an in-memory source string

```bash
cd /tmp/beluna-nanolang-spike/rust_embed
cargo run
```

Result:

- printed `rust_embedded_result=42`
- proved Rust can call the C embedding layer through a wrapper

## Spike Findings

Strong evidence:

- Nanolang can build and run on this local macOS environment.
- The interpreter path can be embedded at C level using exported headers.
- A Rust process can call the embedded C path with a small wrapper.
- The language has syntax that is closer to procedural routine authoring than
  Rhai, especially if Motor wants `async fn` / `await` style routines.

Weak evidence / risks:

- There is no obvious published Rust crate or stable Rust embedding API.
- The embedding proof manually links Nanolang object files.
- The host had to provide process globals such as `g_argc`, `g_argv`, and
  `get_project_root`.
- The C API is broad and compiler-internal-looking; Motor would need to own a
  narrow wrapper boundary.
- Clean routine suspension/resumption across afferent Senses is not proven by
  the simple `inc(41)` spike.
- JSON-like Act construction is awkward unless Motor exposes typed constructors
  or host helper functions.

## Current Read

Nanolang is technically embeddable enough to keep in the candidate set, but not
cheap enough to treat as the default MVP choice.

If the priority is fastest Motor MVP with low integration risk, Rhai is the
better first implementation substrate.

If the priority is Cortex-authored, self-tested, procedure-shaped routines that
can eventually look like `async fn` workflows, Nanolang is the more interesting
language candidate, with the condition that Beluna owns a narrow Rust wrapper
around Nanolang's C runtime.

## Sources

- Nanolang repository and docs: `https://github.com/jordanhubbard/nanolang`
- Rhai book, Rust functions: `https://rhai.rs/book/rust/functions.html`
- Rhai book, object maps: `https://rhai.rs/book/ref/object-maps.html`
- Rhai book, safety limits: `https://rhai.rs/book/engine/options.html`
