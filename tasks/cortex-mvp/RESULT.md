# RESULT - cortex-mvp

- Date: 2026-02-11
- Status: Completed

## Outcome

Cortex was cut over from command-step planning to a reactor-only, always-on loop.

Canonical cortex boundary is now:

`ReactionInput -> CortexReactor -> ReactionResult`

with bounded cognition stages:
1. one primary IR call,
2. bounded subcalls (extractor/filler),
3. deterministic clamp authority,
4. at most one repair,
5. noop fallback when irreparable.

## Implemented Work

1. Reactor-only Cortex module
- Added:
  - `/Users/lanzhijiang/Development/Beluna/core/src/cortex/reactor.rs`
  - `/Users/lanzhijiang/Development/Beluna/core/src/cortex/pipeline.rs`
  - `/Users/lanzhijiang/Development/Beluna/core/src/cortex/clamp.rs`
  - `/Users/lanzhijiang/Development/Beluna/core/src/cortex/adapters/mod.rs`
  - `/Users/lanzhijiang/Development/Beluna/core/src/cortex/adapters/ai_gateway.rs`
- Replaced cortex contracts in:
  - `/Users/lanzhijiang/Development/Beluna/core/src/cortex/types.rs`
  - `/Users/lanzhijiang/Development/Beluna/core/src/cortex/ports.rs`
  - `/Users/lanzhijiang/Development/Beluna/core/src/cortex/error.rs`
  - `/Users/lanzhijiang/Development/Beluna/core/src/cortex/mod.rs`
- Removed step-centric files:
  - `/Users/lanzhijiang/Development/Beluna/core/src/cortex/facade.rs`
  - `/Users/lanzhijiang/Development/Beluna/core/src/cortex/commitment_manager.rs`
  - `/Users/lanzhijiang/Development/Beluna/core/src/cortex/planner.rs`
  - `/Users/lanzhijiang/Development/Beluna/core/src/cortex/state.rs`
  - `/Users/lanzhijiang/Development/Beluna/core/src/cortex/noop.rs`

2. World-relative attempt contract
- Extended `IntentAttempt` with required grounding field:
  - `/Users/lanzhijiang/Development/Beluna/core/src/admission/types.rs`
- Export surface updated:
  - `/Users/lanzhijiang/Development/Beluna/core/src/admission/mod.rs`

3. Runtime protocol + server loop wiring
- Protocol supports ingress event messages:
  - `/Users/lanzhijiang/Development/Beluna/core/src/protocol.rs`
- Server now:
  - constructs real AI gateway-backed cortex adapters,
  - runs `CortexReactor` as a background task,
  - assembles bounded reaction inputs from ingress stream,
  - enforces mechanical backpressure by bounded channels,
  - bridges reactor outbox directly to continuity/admission dispatch,
  - feeds correlated admission outcome signals (`attempt_id` + code) back into cortex ingress context.
  - `/Users/lanzhijiang/Development/Beluna/core/src/server.rs`

4. Config/schema updates
- Added runtime cortex config surface:
  - `/Users/lanzhijiang/Development/Beluna/core/src/config.rs`
  - `/Users/lanzhijiang/Development/Beluna/core/beluna.schema.json`

5. Test migration
- Added new cortex suites:
  - `/Users/lanzhijiang/Development/Beluna/core/tests/cortex/reactor.rs`
  - `/Users/lanzhijiang/Development/Beluna/core/tests/cortex/clamp.rs`
  - `/Users/lanzhijiang/Development/Beluna/core/tests/cortex/ai_gateway_adapter.rs`
- Updated integration flow:
  - `/Users/lanzhijiang/Development/Beluna/core/tests/cortex_continuity_flow.rs`
- Updated admission/continuity constructors for `based_on`:
  - `/Users/lanzhijiang/Development/Beluna/core/tests/admission/admission.rs`
  - `/Users/lanzhijiang/Development/Beluna/core/tests/continuity/debits.rs`
- Removed step/planner tests:
  - `/Users/lanzhijiang/Development/Beluna/core/tests/cortex/commitments.rs`
  - `/Users/lanzhijiang/Development/Beluna/core/tests/cortex/planner.rs`

## Documentation Updates

- `/Users/lanzhijiang/Development/Beluna/docs/features/cortex/PRD.md`
- `/Users/lanzhijiang/Development/Beluna/docs/features/cortex/HLD.md`
- `/Users/lanzhijiang/Development/Beluna/docs/features/cortex/LLD.md`
- `/Users/lanzhijiang/Development/Beluna/docs/contracts/cortex/README.md`
- `/Users/lanzhijiang/Development/Beluna/docs/overview.md`
- `/Users/lanzhijiang/Development/Beluna/core/AGENTS.md`

## Verification

Executed:

```bash
cd /Users/lanzhijiang/Development/Beluna/core
cargo test
```

Result:
- all tests passed.

## Notes

1. Cortex adapters use real AI Gateway at runtime and mocks in tests.
2. Deterministic clamp remains final authority and deterministically drops malformed/unsupported drafts.
