# Core De-Hexagonalization Single-Cutover Plan

## Summary

Remove all hexagonal architecture patterns from `core/` in one cutover by replacing trait-port/adapter indirection with concrete, direct module composition.  
Goal: keep behavior identical while reducing abstraction layers, dynamic dispatch, and "port/adapter/noop" scaffolding.

## Inventory Of Hexagonal Patterns To Remove

1. `spine` port interfaces and runtime polymorphism:

   - `core/src/spine/ports.rs` (`EndpointPort`, `EndpointRegistryPort`, `SpineExecutorPort`)
   - `Arc<dyn ...Port>` usage in `core/src/spine/runtime.rs`, `core/src/spine/router.rs`, `core/src/spine/registry.rs`, `core/src/stem.rs`, `core/src/main.rs`
   - `core/src/spine/noop.rs` (`DeterministicNoopSpine`)
   - `core/src/spine/adapters/*` as architecture “adapter shell” layer (`mod.rs`, `unix_socket.rs`, `catalog_bridge.rs`)

2. `cortex` port interfaces and adapter layer:

   - `core/src/cortex/ports.rs` (`PrimaryReasonerPort`, `AttemptExtractorPort`, `PayloadFillerPort`, `AttemptClampPort`, `CortexPort`, `CortexTelemetryPort`)
   - `core/src/cortex/adapters/*` (`ai_gateway.rs`, `mod.rs`)
   - `Arc<dyn ...Port>` in `core/src/cortex/pipeline.rs`, `core/src/stem.rs`, `core/src/main.rs`
   - `NoopTelemetryPort` pattern in `core/src/cortex/ports.rs`

3. `continuity` port/adapter leftovers:

   - `core/src/continuity/ports.rs` (`SpinePort`)
   - `core/src/continuity/noop.rs` (`SpinePortAdapter`, `NoopDebitSource`)
   - `core/src/continuity/debit_sources.rs` (`ExternalDebitSourcePort` trait abstraction)

4. `ai_gateway` internal port-like abstraction:

   - `core/src/ai_gateway/adapters/mod.rs` (`BackendAdapter` trait + `HashMap<BackendDialect, Arc<dyn BackendAdapter>>`)
   - Trait-object adapter dispatch in `core/src/ai_gateway/gateway.rs`
   - `CredentialProvider` trait-object wiring in gateway construction path (`Arc<dyn CredentialProvider>`)

5. Test-suite hex patterns (must be updated with runtime):

   - Trait-based mocks/fakes in `core/tests/stem/*`, `core/tests/cortex/*`, `core/tests/spine/*`
   - `Arc<dyn ...Port>` test setup throughout the above files

## Target Design (Post-Cutover)

1. Spine becomes concrete:

    - `Spine` owns a concrete `InMemoryEndpointRegistry` and concrete dispatch method.
    - Remove `RoutingSpineExecutor` and call concrete `Spine::dispatch_act(...)` directly.
    - ~Keep Unix socket transport as a normal module under spine, not “adapter architecture”; rename module path to transport-oriented naming (for example `spine/transport/unix_socket.rs`) in same cutover.~
      - 这个不能采纳，保留这层 adapter

2. Cortex becomes concrete pipeline:

   - `CortexPipeline` stores concrete collaborators, not traits.
   - Replace `AIGatewayPrimaryReasoner`/`AIGatewayAttemptExtractor` “adapter”role direct calls into `AIGateway` from pipeline. （建议先重构 AI Gateway）
   - Remove telemetry port trait and use concrete telemetry sink struct/callback type.

3. Continuity stays concrete-only:

- Delete `ports.rs` and `noop.rs`.
- Replace `ExternalDebitSourcePort` trait with concrete source structs consumed directly.

1. ~AI Gateway provider dispatch becomes enum-based concrete dispatch:~ （这个不做，AI Gateway 作为一个 lib 性质的模块，这种程度的抽象是有必要的）

   - Replace `BackendAdapter` trait-object map with `BackendClient` enum (`OpenAiCompatible`, `Ollama`, `GitHubCopilot`).
   - Use `match` dispatch in `gateway.rs`.
   - Replace `CredentialProvider` trait object with a concrete credential resolver struct used directly by gateway.

2. Composition/wiring:

    - `main.rs` creates only concrete structs.
    - `Stem` （从 `StemRuntime` 重命名而来；把那些 `runtime_types` 都放到 `types` 里面，作为全局类型即可 ） holds concrete `Cortex` (自 `CortexRuntime` 重命名而来 ) and `Spine` handles (no `Arc<dyn ...>`).

## Important API / Interface / Type Changes

1. Remove and stop exporting all `*Port` traits in:

   - `core/src/spine/mod.rs`
   - `core/src/cortex/mod.rs`
   - `core/src/continuity/mod.rs`

2. Remove trait-object-returning APIs:

    - `Spine::registry_port() -> Arc<dyn EndpointRegistryPort>` （这个挺好的，没必要移除吧）
    - `Spine::executor_port() -> Arc<dyn SpineExecutorPort>`
    - `global_executor() -> Option<Arc<dyn SpineExecutorPort>>`

3. Replace with concrete APIs:

   - `Spine::registry() -> Arc<InMemoryEndpointRegistry>` (or `&InMemoryEndpointRegistry`)
   - `Spine::dispatch_act(...) -> Result<EndpointExecutionOutcome, SpineError>`

4. Test utility API updates:

   - Remove tests implementing traits just to fake collaborators; replace with concrete fake structs and test-only constructors. （有待考量，有点像虚荣指标了）

## Single-Cutover Implementation Sequence

1. Delete/replace `spine` ports and executor layering:

   - Inline routing logic into `Spine`. （这意味着什么？有点奇怪欸）
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

6. Rewrite tests for concrete constructors and direct collaborator injection.

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

1. Chosen scope: `Everything` (remove all hexagonal patterns, including AI provider adapter traits and transport “adapter architecture” naming).
2. Chosen execution mode: `Single Cutover`.
3. Priority: simplicity over extension points; future re-abstraction is deferred until justified by real variability.
4. Acceptance criterion: no trait-port/adapter/noop architecture layers remain in `core/src`; behavior and test outcomes remain equivalent.
