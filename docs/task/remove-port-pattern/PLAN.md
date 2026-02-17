# Core De-Hexagonalization Single-Cutover Plan

## Summary

Perform a single cutover in `core/` to remove business-path hexagonal runtime indirection (`*Port` trait objects and noop architecture layers), while preserving behavior.
Goal: keep behavior identical while reducing unnecessary abstraction layers and dynamic dispatch in the main runtime path.

## Scope And Inventory

1. `spine` port interfaces and runtime polymorphism:

   - `core/src/spine/ports.rs` (`EndpointPort`, `EndpointRegistryPort`, `SpineExecutorPort`)
   - `Arc<dyn ...Port>` usage in `core/src/spine/runtime.rs`, `core/src/spine/router.rs`, `core/src/spine/registry.rs`, `core/src/stem.rs`, `core/src/main.rs`
   - `core/src/spine/noop.rs` (`DeterministicNoopSpine`)
   - Keep `core/src/spine/adapters/*` as concrete transport/integration modules (not architecture-level adapter shells)

2. `cortex` port interfaces and adapter layer:

   - `core/src/cortex/ports.rs` (`PrimaryReasonerPort`, `AttemptExtractorPort`, `PayloadFillerPort`, `AttemptClampPort`, `CortexPort`, `CortexTelemetryPort`)
   - `core/src/cortex/adapters/*` (`ai_gateway.rs`, `mod.rs`)
   - `Arc<dyn ...Port>` in `core/src/cortex/pipeline.rs`, `core/src/stem.rs`, `core/src/main.rs`
   - `NoopTelemetryPort` pattern in `core/src/cortex/ports.rs`

3. `continuity` port/adapter leftovers:

   - `core/src/continuity/ports.rs` (`SpinePort`)
   - `core/src/continuity/noop.rs` (`SpinePortAdapter`, `NoopDebitSource`)
   - `core/src/continuity/debit_sources.rs` (`ExternalDebitSourcePort` trait abstraction)

4. `ai_gateway` internal library abstractions (out of scope in this cutover):

   - `core/src/ai_gateway/adapters/mod.rs` (`BackendAdapter` trait + `HashMap<BackendDialect, Arc<dyn BackendAdapter>>`)
   - Trait-object adapter dispatch in `core/src/ai_gateway/gateway.rs`
   - `CredentialProvider` trait-object wiring in gateway construction path (`Arc<dyn CredentialProvider>`)

5. Test-suite wiring patterns (update only where required by API changes):

   - Trait-based mocks/fakes in `core/tests/stem/*`, `core/tests/cortex/*`, `core/tests/spine/*` when they block concrete runtime migration
   - `Arc<dyn ...Port>` setup throughout the above files

## Target Design (Post-Cutover)

1. Spine becomes concrete:

   - `Spine` owns a concrete `InMemoryEndpointRegistry` and concrete dispatch method.
   - Remove `RoutingSpineExecutor` and call concrete `Spine::dispatch_act(...)` directly.
   - Keep existing `spine/adapters/*` layout, but treat it as concrete transport/integration code.

2. Cortex becomes concrete pipeline:

   - `CortexPipeline` stores concrete collaborators, not traits.
   - Replace `AIGatewayPrimaryReasoner`/`AIGatewayAttemptExtractor` “adapter” role direct calls into `AIGateway` from pipeline. （建议先重构 AI Gateway）
   - Remove telemetry port trait and use concrete telemetry sink struct/callback type.

3. Continuity stays concrete-only:

   - Delete `ports.rs` and `noop.rs`.
   - Replace `ExternalDebitSourcePort` trait with concrete source structs consumed directly.

4. AI Gateway keeps current internal abstraction in this cutover:

   - Do not force enum-based backend dispatch replacement in this plan.
   - Keep library-level extension seams unless behavior/correctness issues require change.

5. Composition/wiring:

   - `main.rs` creates concrete runtime structs.
   - `Stem` (renamed from `StemRuntime`) holds concrete `Cortex` (renamed from `CortexRuntime`) and `Spine` handles, with no `Arc<dyn ...>` in runtime composition.
   - Move `runtime_types` into a unified `types` module for global runtime types.

## Important API / Interface / Type Changes

1. Remove and stop exporting business-path `*Port` traits in:

   - `core/src/spine/mod.rs`
   - `core/src/cortex/mod.rs`
   - `core/src/continuity/mod.rs`

2. Remove trait-object-returning APIs on runtime hot path:

   - `Spine::registry_port() -> Arc<dyn EndpointRegistryPort>` (replace with concrete registry accessor)
   - `Spine::executor_port() -> Arc<dyn SpineExecutorPort>`
   - `global_executor() -> Option<Arc<dyn SpineExecutorPort>>`

3. Replace with concrete APIs:

   - `Spine::registry() -> Arc<InMemoryEndpointRegistry>` (or `&InMemoryEndpointRegistry`)
   - `Spine::dispatch_act(...) -> Result<EndpointExecutionOutcome, SpineError>`

4. Test API updates:

   - Apply minimal test rewrites required by constructor/signature changes.
   - Do not refactor tests solely for style.

## Single-Cutover Implementation Sequence

1. Delete/replace `spine` ports and executor layering:

   - Remove `RoutingSpineExecutor` indirection and expose concrete `Spine::dispatch_act(...)`.
   - Keep routing semantics intact.
   - Update registry and body endpoint registration to use concrete types.

2. Rewire `stem` to call concrete `Spine` and concrete `Cortex`.

   - Remove `Arc<dyn CortexPort>` and `Arc<dyn SpineExecutorPort>` fields from `core/src/stem.rs`.

3. Flatten `cortex`:

   - Move behavior from `cortex/adapters/ai_gateway.rs` into concrete cortex collaborators or into `CortexPipeline`.
   - Replace trait calls with concrete method calls.

4. Remove continuity hex leftovers:

   - Delete `continuity/ports.rs` and `continuity/noop.rs`.
   - Replace debit source trait usage with direct concrete fields.

5. Update module exports and imports (`mod.rs` files and callers).

6. Update tests only where constructor/signature changes require it, preserving existing behavior assertions.

7. Run full verification and fix compile/test regressions.

## Test Cases And Scenarios

1. Build/compile gates:

   - `cargo test --manifest-path core/Cargo.toml --no-run`

2. Full runtime regression suite:

   - `cargo test --manifest-path core/Cargo.toml`

3. Focused behavior suites that must remain green:

   - `core/tests/stem_bdt.rs`
   - `core/tests/spine_bdt.rs`
   - `core/tests/cortex_bdt.rs`
   - `core/tests/continuity_bdt.rs`
   - `core/tests/ai_gateway_bdt.rs`

4. Scenario parity checks:

   - Spine dispatch: missing endpoint, endpoint error mapping, applied/rejected/deferred conversion
   - Stem loop: sleep handling, capability patch/drop handling, serial dispatch order
   - Cortex pipeline: timeout/budget/fallback behavior unchanged
   - AI gateway routing/reliability/retry/tool behavior unchanged

## Risks And Mitigations

1. Large blast radius from single cutover:

   - Mitigation: keep behavior-lock tests unchanged first, then refactor until all green.

2. Hidden dead-code coupling:

   - Mitigation: strict compile-first checkpoints after each major module rewrite.

3. Public re-export breakage:

   - Mitigation: update all internal call sites in same commit and remove obsolete exports atomically.

## Assumptions And Defaults

1. Chosen scope: `Selective de-hexagonalization` (remove business-path `*Port` trait-object/noop architecture layers in `spine`, `cortex`, `continuity`; keep AI Gateway library-level abstractions and existing transport adapter layout where reasonable).
2. Chosen execution mode: `Single Cutover`.
3. Priority: simplicity over extension points; future re-abstraction is deferred until justified by real variability.
4. Acceptance criterion: no business-path `*Port` trait-object/noop architecture layers remain in `core/src` runtime composition; behavior and test outcomes remain equivalent.
