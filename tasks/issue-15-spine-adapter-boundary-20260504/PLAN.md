# Issue 15 - Spine Runtime And Adapter Boundary

## MVT Core

- Objective & Hypothesis: Clarify and implement the Spine runtime / body endpoint adapter interface, topology, sequencing, and adapter config ownership while preserving the existing JSON shape under `spine.adapters`.
- Guardrails Touched: Core remains the typed config shape authority; Spine dispatch remains a mechanical `act.endpoint_id` route lookup; adapter-local transport, protocol, mailbox, retry, timeout, and config concerns stay inside adapter boundaries.
- Verification: Rust type ownership shows adapter configs under adapter modules; shared Spine runtime has no field-level knowledge of adapter configs; generated schema still exposes `spine.adapters`; targeted `cargo test --manifest-path core/Cargo.toml` checks pass.

## Exploration Scaffold

- Perturbation: GitHub issue #15 asks to clarify Spine runtime / body endpoint adapter boundaries and move adapter-specific config ownership into adapters.
- Input Type: Constraint plus Artifact.
- Active Mode or Transition Note: Explore. This packet records current evidence and slices before production code or durable docs are changed.
- Governing Anchors:
  - `AGENTS.md`
  - `core/AGENTS.md`
  - `core/src/spine/AGENTS.md`
  - `core/src/body/AGENTS.md`
  - `docs/20-product-tdd/cross-unit-contracts.md`
  - `docs/20-product-tdd/system-state-and-authority.md`
  - `docs/30-unit-tdd/core/design.md`
  - `docs/30-unit-tdd/core/verification.md`
- Impact Hypothesis: A sharper adapter boundary should reduce incidental coupling in shared Spine runtime code, make adapter addition safer, and keep endpoint-specific concerns close to their transport or mailbox implementation.
- Temporary Assumptions:
  - Keep the external config shape `spine.adapters[]` with `{ type, config }`.
  - No backward compatibility work is required beyond preserving the current config shape.
  - Built-in inline body endpoints may remain in `core/src/body` during the first implementation slice.
- Negotiation Triggers:
  - Changing endpoint wire protocol semantics.
  - Moving built-in inline endpoint startup ownership out of `main`.
  - Splitting `Spine` into separate dispatch-kernel and adapter-supervisor structs.
  - Updating Product TDD or Unit TDD with durable boundary claims.
- Promotion Candidates:
  - Final runtime / adapter interface contract.
  - Final startup and shutdown topology.
  - Adapter config ownership rule.
  - Inline body endpoint composition root rule if the bad smell is fixed or explicitly accepted.

## Current Topology And Sequencing

### Core Startup

1. `main` loads typed `Config`.
2. `main` initializes observability and Stem/Cortex dependencies.
3. `main` calls `Spine::new(&config.spine, afferent_ingress, stem_control)`.
4. `Spine::new` constructs a `Spine` value and immediately calls `start_adapters`.
5. `Spine::start_adapters` iterates `config.spine.adapters`, assigns adapter ids, constructs each concrete adapter, stores the inline adapter in `OnceLock`, and spawns unix-socket adapter tasks.
6. `main` fetches `spine_runtime.inline_adapter()`.
7. `main` reads `config.body.std_shell` and `config.body.std_web`, then calls `start_inline_body_endpoints`.
8. Each enabled inline body endpoint starts a dedicated thread, attaches itself to the inline adapter, receives `act_rx`, and emits result senses through `sense_tx`.

### Runtime Dispatch

1. Stem calls `Spine::on_act_final`.
2. `Spine` performs a mechanical route lookup by endpoint id.
3. Inline dispatch calls an inline adapter proxy, which enqueues the act into the endpoint mailbox.
4. Unix-socket dispatch sends the act to an adapter channel opened by a socket session.
5. Endpoint execution feedback returns asynchronously as sense ingress.

### Adapter Ingress

1. Inline adapter receives endpoint-produced `InlineSenseDatum`, injects the registered runtime endpoint id, clamps weight, and publishes the sense to Spine.
2. Unix-socket adapter parses NDJSON messages, authenticates/registers remote endpoints, namespaces proprioception keys, injects authenticated endpoint id into senses, and publishes senses to Spine.

### Shutdown

1. Core eventually calls `shutdown_global_spine(spine)`.
2. `Spine::shutdown` cancels the shared token and awaits adapter tasks stored in `Spine.tasks`.
3. Inline endpoint sense tasks observe the same cancellation token through the inline adapter.

## Observed Coupling And Bad Smells

### Adapter Config Ownership Leakage

- `core/src/config/spine.rs` defines adapter-specific structs, defaults, validation, and enum variants for inline and unix-socket adapters.
- `core/src/config.rs` pattern-matches `SpineAdapterConfig::UnixSocketNdjson` to normalize `socket_path`.
- `core/src/spine/runtime.rs` reads `act_queue_capacity`, `sense_queue_capacity`, and `socket_path` directly while starting adapters.

### Runtime / Adapter Lifecycle Mixing

- `Spine` contains dispatch routing state, endpoint registration state, adapter task lifecycle, shutdown token, and inline adapter discovery.
- Adapter task ownership living in `Spine.tasks` makes `Spine` both dispatch kernel and adapter supervisor.
- This keeps shutdown simple, but it weakens the conceptual boundary between mechanical dispatch and concrete adapter lifecycle.

### Inline Body Endpoint Composition Root Bad Smell

- `main` knows the built-in endpoint inventory: shell and web.
- `main` reads `config.body.std_shell.*` and `config.body.std_web.*`, then passes booleans and limits into `start_inline_body_endpoints`.
- This means Core composition root currently owns inline endpoint startup policy and endpoint-specific config shape.
- The first issue-15 slice can record this as a known smell while keeping it stable. A later slice can move built-in inline endpoint startup behind a body-owned startup plan or body-owned config interpreter.

### Test Harness Drift

- `core/tests/agent-task/kit/runner.rs` constructs `SpineRuntimeConfig` manually with adapter-specific variants.
- One harness branch uses an empty adapter list even though runtime config validation rejects empty adapter arrays.

## Meaning Of Variant Selection Versus Field-Level Knowledge

`Spine::start_adapters` currently does two different jobs:

1. It chooses which adapter kind to start from the `type` tag.
2. It interprets the fields inside that adapter's `config` block.

Keeping variant selection means shared runtime may still branch on adapter kind, for example `inline` versus `unix-socket-ndjson`, because it is the local composition point for runtime startup.

Removing field-level knowledge means shared runtime should not know that inline has `act_queue_capacity` or that unix socket has `socket_path`. Those details should be interpreted by adapter-owned constructors or adapter-owned starter functions.

This is the shallowest acceptable implementation slice. A deeper slice would also remove adapter task supervision and inline adapter discovery from `Spine`, placing them in an explicit adapter supervisor or runtime host.

## Proposed Slice Plan

### Slice 0 - Packet And Agreement

Goal: Record the current boundary map, smells, and implementation slices.

Exit criteria:
- This task packet exists.
- Human confirms which slice enters Execute first.

### Slice 1 - Adapter Config Ownership With Current Topology

Goal: Move adapter-specific config structs, defaults, validation helpers, and path normalization into adapter-owned modules while keeping current `Spine` topology and JSON shape.

Status: implemented in the first Slice 1 pass.

Likely files:
- `core/src/config/spine.rs`
- `core/src/config.rs`
- `core/src/spine/adapters/inline.rs` or `core/src/spine/adapters/inline/config.rs`
- `core/src/spine/adapters/unix_socket.rs` or `core/src/spine/adapters/unix_socket/config.rs`
- `core/src/spine/adapters/mod.rs`
- `core/src/spine/runtime.rs`
- `core/beluna.schema.json`
- `core/tests/agent-task/kit/runner.rs`

Verification:
- `rg` should show adapter config definitions in adapter modules.
- `Config::normalize_paths` should delegate Spine adapter path normalization instead of matching unix-socket fields directly.
- Generated schema still contains `spine.adapters` with inline and unix-socket forms.
- `cargo test --manifest-path core/Cargo.toml`.

Result:
- Added adapter-owned config modules under `core/src/spine/adapters/inline/` and `core/src/spine/adapters/unix_socket/`.
- Moved inline queue defaults and unix-socket path default/validation/normalization into those adapter-owned modules.
- Changed root config path normalization to delegate through `SpineRuntimeConfig`.
- Changed Spine runtime startup to call adapter-owned APIs for inline construction/start emission and unix-socket task spawning.
- Regenerated `core/beluna.schema.json`; it produced no schema diff.
- Ran `cargo fmt --manifest-path core/Cargo.toml -- --check`.
- Ran `cargo check --manifest-path core/Cargo.toml`.
- Ran `cargo test --manifest-path core/Cargo.toml`.

### Slice 2 - Runtime / Adapter Interface And Topology

Goal: Make the runtime / adapter interface explicit in code so Spine dispatch state and adapter lifecycle responsibilities are easier to distinguish.

Status: implemented in the first Slice 2 pass.

Candidate shapes:
- Conservative: keep `Spine` as the object that owns adapter lifecycle, but introduce adapter-owned starter APIs and a narrow `AdapterContext`.
- Stronger: introduce an explicit adapter host/supervisor that owns adapter tasks, cancellation token, and inline adapter discovery; `Spine` owns dispatch, endpoint registry, sense publishing, and proprioception updates.

Discussion decisions:
- Spine Runtime dispatch should stop at the adapter level.
- Adapter session/channel selection belongs inside the adapter boundary.
- Runtime / adapter data-plane ownership should be explicit:
  - Spine Runtime owns adapter-level `act_tx` handles and adapter-level `sense_rx` handles.
  - Adapter owns adapter-level `act_rx` and adapter-level `sense_tx`.
  - Adapter may own adapter-internal body endpoint session or mailbox details behind that adapter-level boundary.
- Introduce `AdapterContext` and `SpineAdapterPort` for adapter control-plane operations.
- Rename `refresh_topology_proprioception` to `publish_topology_proprioception_snapshot`.
- Retire `on_adapter_channel_open/closed` as a Spine Runtime interface; unix-socket session open/closed should stay adapter-local.

Likely files:
- `core/src/spine/runtime.rs`
- `core/src/spine/adapters/mod.rs`
- `core/src/spine/adapters/inline.rs`
- `core/src/spine/adapters/unix_socket.rs`
- `core/src/main.rs`
- targeted tests under `core/tests/*` or local inline tests if the invariant is implementation-local.

Verification:
- Shared dispatch path still returns `Acknowledged`, `Rejected`, or `Lost`.
- Adapter shutdown behavior remains bounded.
- Inline endpoints still attach through inline adapter and emit senses.
- Unix-socket endpoint registration and act ack behavior remain intact.

Result:
- Added explicit adapter-level data-plane ownership:
  - Spine Runtime stores adapter-level `act_tx` handles and owns adapter-level `sense_rx` receive loops.
  - Each adapter receives `act_rx` and `sense_tx` through `AdapterContext`.
- Added `SpineAdapterPort` as the adapter control-plane boundary for endpoint registration, descriptor patch/drop, endpoint drop, proprioception patch/drop, and topology snapshot publication.
- Replaced endpoint dispatch bindings with adapter-level bindings; adapter-internal session/channel/mailbox selection now stays behind the adapter boundary.
- Removed Spine Runtime adapter-channel registry concepts and retired `on_adapter_channel_open/closed`.
- Renamed `refresh_topology_proprioception` to `publish_topology_proprioception_snapshot`.
- Updated inline adapter startup to `SpineInlineAdapter::from_config(config, AdapterContext)`.
- Updated unix-socket adapter startup to `spawn_adapter_task(config, AdapterContext)`.
- Adapter-local stale mailbox/session misses now trigger endpoint drops through `SpineAdapterPort`.
- Added `drop_ns_descriptors` to the adapter port so adapters can explicitly drop route subsets through the same descriptor authority path.
- Updated agent-task ack endpoint setup to attach through the inline adapter boundary.
- Added replay AIMock journal summary evidence to classify fixture/tool matching failures.

Diagnostic note:
- `core.intent_to_act_ack.v1` initially failed after the topology change because the harness moved only `act_rx` into the ack endpoint task; Rust 2021 disjoint field capture dropped `sense_tx`, causing the inline adapter sense task to remove the endpoint before Cortex tick. The harness task now holds `sense_tx` for the endpoint task lifetime.

Verification result:
- `rg -n "EndpointBinding::Inline|EndpointBinding::AdapterChannel|AdapterChannel|adapter_channel|adapter_channels|by_channel|on_adapter_channel|refresh_topology_proprioception|next_adapter_channel|remove_ns_descriptors" core/src core/tests -g '*.rs'` produced no matches.
- `cargo fmt --manifest-path core/Cargo.toml --check`.
- `cargo check --manifest-path core/Cargo.toml`.
- `cargo test --manifest-path core/Cargo.toml`.

### Slice 3 - Inline Body Endpoint Startup Smell

Goal: Decide whether built-in inline endpoint startup should remain in `main` or move behind a body-owned startup plan/config interpreter.

Candidate shape:
- `main` passes `config.body` and `Arc<SpineInlineAdapter>` into a body-owned function.
- `core/src/body` interprets endpoint enablement, limits, feature gating, and startup ordering.
- `main` stops knowing shell/web endpoint inventory.

Likely files:
- `core/src/body/mod.rs`
- `core/src/config/body.rs`
- `core/src/main.rs`
- `core/src/body/AGENTS.md`
- tests touching agent-task std-shell startup.

Verification:
- Existing std-shell agent-task replay still works.
- Feature-disabled errors remain explicit and local to the body endpoint startup boundary.

### Slice 4 - Durable Docs Promotion

Goal: Promote stable boundary claims after implementation proves them.

Candidate destinations:
- `core/src/spine/AGENTS.md`
- `core/src/body/AGENTS.md`
- `docs/30-unit-tdd/core/design.md`
- `docs/30-unit-tdd/core/interfaces.md`
- `docs/30-unit-tdd/core/verification.md`
- Product TDD only if cross-unit contract semantics change.

Verification:
- Durable docs describe the implemented topology, not a target architecture.

## Open Questions

1. Should Slice 1 stop at config ownership, or should Slice 2's adapter host split happen in the same PR?
2. Should `Spine` remain the public dispatch object name, with an adapter supervisor hidden behind it, or should the code introduce a new public lifecycle object such as `SpineRuntime`?
3. Should inline adapter discovery remain `spine.inline_adapter()`, or should built-in body endpoint startup receive an adapter handle from an adapter host?
4. Should the agent-task harness go through validated config construction, or is manual construction acceptable for test-local setup?
