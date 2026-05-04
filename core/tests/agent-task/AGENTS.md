# AGENTS.md of Core Agent Task Tests

Agent Task Tests evaluate whether Beluna Core can complete business-level agent tasks through the real Core path.
This file is both the local policy and the operating guide for `core/tests/agent-task/`.

## Purpose

Agent Task Tests cover the loop from user intent through afferent admission, Cortex reasoning, efferent dispatch, Spine routing, endpoint feedback, and task-level verification.

They are the preferred Core verification layer when a change can affect a real agent task. They complement narrower integration tests and local inline tests.

## Directory Shape

```text
core/tests/agent-task/
├── main.rs
├── kit/
│   ├── ai.rs
│   ├── case.rs
│   ├── endpoints.rs
│   ├── evidence.rs
│   ├── o11y.rs
│   ├── oracle.rs
│   ├── runner.rs
│   └── workspace.rs
└── cases/
    └── <case-id>/
        ├── case.yaml
        └── fixtures/
            └── llm.json
```

## Modes

- `replay`: deterministic AIMock-backed execution. Use this for local regression and CI.
- `live`: real configured AI Gateway and real LLM execution. Use this for capability calibration and model-facing design diagnosis.

Replay should pass before treating a case as stable. Live can pass, fail, or drift; its value is in the artifact trail.

## Commands

Replay all Agent Task Cases:

```bash
cargo test --manifest-path core/Cargo.toml --test agent_task -- --nocapture
```

Run the full Core test surface:

```bash
cargo test --manifest-path core/Cargo.toml
```

Run one live Agent Task Case:

```bash
set -a
source .env
set +a
BELUNA_AGENT_TASK_CONFIG=/Users/lanzhijiang/Development/Beluna/beluna.jsonc \
BELUNA_AGENT_TASK_CASE=core.shell_write_file_world_diff.v1 \
cargo test --manifest-path core/Cargo.toml --test agent_task -- --ignored --nocapture
```

Live runs are ignored by default and should stay opt-in because they spend model budget and can vary by backend.

## Artifacts

Run artifacts are written under:

```text
core/target/agent-task-runs/<mode>/<case-id>/<run-id>/
```

Expected artifacts include:

- `result.json`: pass/fail, wall time, tick count, event counts
- `evidence.jsonl`: harness evidence stream
- `world-before.json`, `world-after.json`, `world-diff.json`: workspace oracle artifacts
- `o11y-contract-events.jsonl`: live `observability.contract` events when live capture is enabled
- `ai-gateway-summary.json`: live AI Gateway request and chat-turn summary

When a live run fails, inspect artifacts before changing prompts, descriptors, endpoint act shapes, or tick budgets.

## Case Design

Each case should have one primary proof.

Examples:

- `core.intent_to_act_ack.v1`: primary proof is the user intent to acknowledged act path.
- `core.shell_write_file_world_diff.v1`: primary proof is workspace file state.

Use secondary evidence for diagnosis:

- act dispatch outcome
- emitted endpoint senses
- world diff
- AI Gateway turn transcript
- Cortex, Stem, or Spine contract events

## Oracle Rules

- Prefer task-level world state or public evidence over implementation details.
- Use `content_exact` when byte-level content matters.
- Use `content_trimmed_exact` when terminal newline differences are outside the task risk.
- Use correlation checks when act/sense identity matters.
- Keep endpoint internals out of the primary oracle unless the case targets endpoint integration.

## Replay Fixtures

Replay fixtures should represent a plausible model behavior path and should stay small.

Use fixture placeholders when a run-specific value is needed:

- `$CASE_WORKSPACE`
- `$CASE_TMP`

Do not encode volatile run ids, timestamps, or absolute artifact paths directly into fixtures.

## Live Rules

Live mode exists to test Core's real model-facing shape:

- prompt and IR clarity
- tool schemas and descriptor names
- endpoint act payload ergonomics
- sense feedback usefulness
- tick and model-call budgets

Live failures are design evidence. Classify the failure before patching:

- model did not call the needed tool
- model called the wrong tool
- tool payload shape was hard for the model
- endpoint result sense was insufficient for correction
- tick budget ended before correction
- world boundary was unclear
- backend or credential failed

## Adding A Case

Checklist:

- Name the case as `<unit>.<capability>.<version>`, for example `core.shell_write_file_world_diff.v1`.
- Define the user intent and success claim in `case.yaml`.
- Materialize only the world state the task needs.
- Choose exactly one primary oracle.
- Add diagnostic evidence only when it helps classify failures.
- Add deterministic AIMock replay fixtures.
- Run replay locally.
- Run live when the case targets model-facing behavior.
- Update task packet notes when the case reveals reusable design knowledge.

## Adding Kit Primitives

Add a kit primitive only after at least one real case needs it and the concept is simpler than handwritten test code.

Good primitive examples:

- workspace materialization
- file tree snapshots and diffs
- contract event capture
- generic evidence matching
- common file content expectations

Avoid case-specific helper names in the kit. The kit should expose primitive operations that future cases can combine.
