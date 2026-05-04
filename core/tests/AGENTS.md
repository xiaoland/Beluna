# AGENTS.md of Core Tests

Core tests are a business-capability verification layer for Beluna Core.
They should preserve confidence in Core behavior, not inflate coverage counts.

## Test Placement

- `core/tests/agent-task/`: black-box Agent Task Cases. Use these for user-intent-to-world or user-intent-to-act capabilities that cross afferent, Cortex, efferent, Spine, body endpoint, continuity, or observability boundaries.
- Other `core/tests/*`: black-box integration tests over public Core APIs or cross-module contracts. Keep them focused on stable boundaries.
- Inline tests near implementation: white-box, fine-grained invariants tied to a local implementation detail. Keep them rare and easy to delete when the implementation changes.

## When To Add Tests

Add or update an Agent Task Case when a change affects any of these:

- task success from user intent through Core orchestration
- Cortex prompt, IR shape, tool schema, descriptor catalog, or endpoint act shape
- afferent/efferent ordering, dispatch feedback, or sense propagation
- workspace/world state outcomes
- live LLM ability to use a Core capability

Add an integration test under `core/tests/*` when the expected behavior is a public contract and the proof does not need a full Agent Task loop.

Add an inline test only when the invariant is local, stable, and cheaper to understand next to the code than through a black-box test.

## Test Quality Rules

- Prefer one primary proof per test.
- Prefer world state, public API state, or contract evidence over internal implementation assertions.
- Keep diagnostics as artifacts or secondary evidence unless the case explicitly targets that path.
- Keep replay tests deterministic enough for routine local and CI use.
- Use live tests for capability calibration, model-facing design diagnosis, and release confidence.
- When deleting stale tests, preserve the business risk as a new Agent Task Case only if the risk still matters.

## Commands

```bash
cargo test --manifest-path core/Cargo.toml --test agent_task -- --nocapture
cargo test --manifest-path core/Cargo.toml
```

Live Agent Task Tests require explicit environment and are ignored by default. See `core/tests/agent-task/AGENTS.md`.
