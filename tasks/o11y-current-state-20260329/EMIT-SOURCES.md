# Emit Sources (Who Initiates What)

This file maps each contract event family to the real initiating code path in `core/src`.

## Common Emission Chain

Business/runtime code
-> `observability_runtime::emit_*` call site
-> wrapper constructor in `core/src/observability/runtime/*.rs`
-> `emit_contract_event` in `core/src/observability/runtime/emit.rs`
-> `tracing::{info,warn}!` with target `observability.contract` and message `contract_event`

## ai-gateway Subsystem

### family: ai-gateway.request

Initiator:

- `ChatRuntime::dispatch_complete` in `core/src/ai_gateway/chat/runtime.rs`

Key call sites:

- start: `core/src/ai_gateway/chat/runtime.rs:85`
- failed (backend disallowed): `core/src/ai_gateway/chat/runtime.rs:123`
- succeeded: `core/src/ai_gateway/chat/runtime.rs:182`
- attempt_failed: `core/src/ai_gateway/chat/runtime.rs:257`
- failed (retry exhausted): `core/src/ai_gateway/chat/runtime.rs:293`

Wrapper:

- `core/src/observability/runtime/ai_gateway.rs:31` (`emit_ai_gateway_request`)

### family: ai-gateway.turn

Initiator:

- `Thread::complete` in `core/src/ai_gateway/chat/thread.rs`

Key call sites:

- error path: `core/src/ai_gateway/chat/thread.rs:146`
- committed/committed_pending_continuation: `core/src/ai_gateway/chat/thread.rs:234`

Wrapper:

- `core/src/observability/runtime/ai_gateway.rs:74` (`emit_ai_gateway_turn`)

### family: ai-gateway.thread

Initiator:

- `emit_thread_snapshot_event` in `core/src/ai_gateway/chat/api_chat.rs`
- `Thread::complete` in `core/src/ai_gateway/chat/thread.rs`

Key call sites:

- opened/cloned snapshot: `core/src/ai_gateway/chat/api_chat.rs:226`
- turn_committed snapshot: `core/src/ai_gateway/chat/thread.rs:257`

Wrapper:

- `core/src/observability/runtime/ai_gateway.rs:107` (`emit_ai_gateway_thread`)

## cortex Subsystem

### family: cortex.tick

Initiator:

- `Cortex::cortex` in `core/src/cortex/runtime/primary.rs`

Key call site:

- `core/src/cortex/runtime/primary.rs:920`

Trigger notes:

- emitted from `emit_tick_observation` closure
- status currently: `ok` and `primary_contract_error`

Wrapper:

- `core/src/observability/runtime/cortex.rs:14` (`emit_cortex_tick`)

### family: cortex.organ

Initiator:

- `Cortex::run_primary_turn` and `Cortex::run_organ` in `core/src/cortex/runtime/primary.rs`

Key call sites:

- start: `core/src/cortex/runtime/primary.rs:1286`, `core/src/cortex/runtime/primary.rs:1393`
- end(error): `core/src/cortex/runtime/primary.rs:1311`, `core/src/cortex/runtime/primary.rs:1446`, `core/src/cortex/runtime/primary.rs:1463`
- end(ok): `core/src/cortex/runtime/primary.rs:1335`, `core/src/cortex/runtime/primary.rs:1486`

Wrapper:

- `core/src/observability/runtime/cortex.rs:41` (`emit_cortex_organ_start`)
- `core/src/observability/runtime/cortex.rs:68` (`emit_cortex_organ_end`)

### family: cortex.goal-forest

Initiator:

- snapshot in `Cortex::cortex`
- patch in internal tool execution flow (goal forest patch)

Key call sites:

- snapshot: `core/src/cortex/runtime/primary.rs:916`
- patch: `core/src/cortex/runtime/primary.rs:596`, `core/src/cortex/runtime/primary.rs:613`

Wrapper:

- `core/src/observability/runtime/cortex.rs:99` (`emit_cortex_goal_forest_snapshot`)
- `core/src/observability/runtime/cortex.rs:127` (`emit_cortex_goal_forest_patch`)

## stem Subsystem

### family: stem.tick

Initiator:

- `StemTickRuntime::run` in `core/src/stem/runtime.rs`

Key call site:

- `core/src/stem/runtime.rs:377`

Wrapper:

- `core/src/observability/runtime/stem.rs:11` (`emit_stem_tick`)

### family: stem.signal

Initiator:

- afferent pathway scheduler
- efferent dispatch runtime

Key call sites:

- enqueue (afferent): `core/src/stem/afferent_pathway.rs:360`
- defer (afferent): `core/src/stem/afferent_pathway.rs:509`
- release (afferent): `core/src/stem/afferent_pathway.rs:670`
- dispatch (efferent): `core/src/stem/efferent_pathway.rs:248`
- result (efferent): `core/src/stem/efferent_pathway.rs:346`

Wrapper:

- `core/src/observability/runtime/stem.rs:23` (`emit_stem_signal_transition`)

### family: stem.dispatch

Initiator:

- producer enqueue path
- efferent dispatch runtime

Key call sites:

- enqueue: `core/src/stem/efferent_pathway.rs:82`
- dispatch: `core/src/stem/efferent_pathway.rs:266`
- result: `core/src/stem/efferent_pathway.rs:364`

Wrapper:

- `core/src/observability/runtime/stem.rs:69` (`emit_stem_dispatch_transition`)

### family: stem.proprioception

Initiator:

- state store mutation paths in `StemPhysicalStateStore`

Key call sites:

- patch: `core/src/stem/runtime.rs:234`
- drop: `core/src/stem/runtime.rs:249`

Upstream callers (indirect initiators):

- efferent status patch/drop (`core/src/stem/efferent_pathway.rs`)
- spine adapter topology/state updates via `StemControlPort` (`core/src/spine/adapters/unix_socket.rs`)

Wrapper:

- `core/src/observability/runtime/stem.rs:100` (`emit_stem_proprioception`)

### family: stem.descriptor.catalog

Initiator:

- descriptor patch/drop commit in `StemPhysicalStateStore`

Key call sites:

- patch commit: `core/src/stem/runtime.rs:134`
- drop commit: `core/src/stem/runtime.rs:207`

Upstream callers:

- endpoint descriptor add/remove in `core/src/spine/runtime.rs`

Wrapper:

- `core/src/observability/runtime/stem.rs:111` (`emit_stem_descriptor_catalog`)

### family: stem.afferent.rule

Initiator:

- rule command handling in `handle_command` (`add`/`remove`) in `core/src/stem/afferent_pathway.rs`

Key call sites:

- add: `core/src/stem/afferent_pathway.rs:574`
- remove: `core/src/stem/afferent_pathway.rs:607`

Wrapper:

- `core/src/observability/runtime/stem.rs:134` (`emit_stem_afferent_rule`)

## spine Subsystem

### family: spine.adapter

Initiator:

- adapter lifecycle in `Spine::start_adapters`

Key call sites:

- inline enabled: `core/src/spine/runtime.rs:189`
- unix enabled: `core/src/spine/runtime.rs:219`
- unix faulted: `core/src/spine/runtime.rs:228`

Wrapper:

- `core/src/observability/runtime/spine.rs:8` (`emit_spine_adapter_lifecycle`)

### family: spine.endpoint

Initiator:

- endpoint registration and removal in `Spine`

Key call sites:

- connected on add: `core/src/spine/runtime.rs:499`
- dropped on remove: `core/src/spine/runtime.rs:635`

Wrapper:

- `core/src/observability/runtime/spine.rs:26` (`emit_spine_endpoint_lifecycle`)

### family: spine.dispatch

Initiator:

- binding and outcome in act dispatch flow (`Spine::dispatch_act`, `Spine::log_dispatch_outcome`)

Key call sites:

- bind: `core/src/spine/runtime.rs:343`, `core/src/spine/runtime.rs:379`
- outcome: `core/src/spine/runtime.rs:813`

Wrapper:

- `core/src/observability/runtime/spine.rs:46` (`emit_spine_dispatch_bind`)
- `core/src/observability/runtime/spine.rs:73` (`emit_spine_dispatch_outcome`)
