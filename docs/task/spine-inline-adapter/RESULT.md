# Spine Inline Adapter Result
- Task Name: `spine-inline-adapter`
- Date: `2026-02-18`
- Status: `COMPLETED`

## 1) Summary
Implemented Spine Inline Adapter as an adapter-owned mailbox runtime for Core-bundled inline body endpoints.

Final dispatch behavior for inline endpoints is:
1. `Act -> Enqueue -> End` (no endpoint ACK channel).
2. Execution outcomes are reflected via emitted `Sense`, not synchronous dispatch return values.
3. Inline endpoint threads are started by `main` from `config.body`, while adapters are started by Spine Runtime from `spine.adapters`.

## 2) Implemented Changes

### Spine config/schema
1. Added `spine.adapters` variant `type: "inline"` with config:
   - `act_queue_capacity`
   - `sense_queue_capacity`
2. Updated default adapter list to include inline adapter plus unix-socket adapter.
3. Updated schema validation to support `oneOf` adapter forms (`inline`, `unix-socket-ndjson`).

### Spine runtime + adapter
1. Added `core/src/spine/adapters/inline.rs`:
   - `SpineInlineAdapter`
   - adapter-owned endpoint mailbox lifecycle
   - `attach_inline_endpoint(...)`
   - endpoint proxy routing (`EndpointBinding::Inline` target is adapter proxy)
   - capability patch/drop sense emission on attach/detach
2. Runtime now starts inline adapter from `spineConfig.adapters`.
3. Runtime now exposes `Spine::inline_adapter()` for `main` to pass into inline endpoint startup.

### Inline body endpoints
1. Reworked `core/src/body/mod.rs`:
   - startup now takes `Arc<SpineInlineAdapter>`
   - each endpoint runs in a dedicated named thread
   - endpoint thread attaches to inline adapter at startup
   - startup failure on attach/register now fails Core startup
2. Endpoint act handling now consumes adapter mailbox (`act_rx`) and emits sense through adapter mailbox (`sense_tx`).

### Runtime wiring
1. `core/src/main.rs` now requires configured inline adapter when inline body endpoints are enabled.
2. `main` no longer passes pathway/spine direct registration handles into body module.

### Docs
1. Added/updated docs for new ownership and startup model:
   - `docs/task/spine-inline-adapter/PLAN.md`
   - `core/src/body/AGENTS.md`
   - `core/src/spine/AGENTS.md`
   - `docs/modules/spine/README.md`
   - `core/AGENTS.md`

## 3) Validation
Commands run in `core/`:
1. `cargo check`
2. `cargo test spine::adapters::inline::tests:: --lib`
3. `cargo test spine::runtime::tests:: --lib`
4. `cargo test body::shell::tests:: --lib`
5. `cargo test --test spine_bdt dispatch`
6. `cargo test --test stem_bdt dispatch_pipeline`

Results:
1. all above commands passed
2. new inline adapter unit test passed (`attach_registers_endpoint_and_forwards_sense`)

## 4) Notes
1. Spine dispatch surface now uses `ActDispatchResult` at endpoint/adapter dispatch boundary.
2. Inline dispatch success is determined by enqueue success into endpoint act mailbox.
