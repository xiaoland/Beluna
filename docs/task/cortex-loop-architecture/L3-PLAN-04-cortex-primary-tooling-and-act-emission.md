# L3 Plan 04 - Cortex Primary Tooling and Act Emission
- Task: `cortex-loop-architecture`
- Micro-task: `04-cortex-primary-tooling-and-act-emission`
- Stage: `L3`
- Date: `2026-03-01`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Objective
Implement dynamic per-act tool emission in Cortex Primary and bounded `wait_for_sense` semantics powered by afferent-pathway deferral control.

## 2) Execution Steps
### Step 1 - Contract and Limits Reshape
1. Update `core/src/cortex/types.rs`:
- add `ReactionLimits.max_waiting_seconds`.
- replace `CortexOutput { acts, wait_for_sense: bool }` with emitted-act envelope contract:
  - `emitted_acts: Vec<EmittedAct>`
  - `control: CortexControlDirective`
2. Keep external tool parameter name `wait_for_sense`, but use internal field name `wait_for_sense_seconds`.
3. Update serde defaults for new limits and output contract.

### Step 2 - Dynamic Act Tool Generation
1. In `core/src/cortex/primary.rs`, generate one tool per act descriptor for each turn.
2. Tool name is transport-safe alias (for example `act_0001`) with deterministic map to fq act id (`endpoint-id/neural-signal-descriptor-id`).
3. Tool input schema fields:
- `payload` (descriptor schema)
- `wait_for_sense` (`integer >= 0`)
4. Build per-turn tool overrides from physical-state act descriptor snapshot.

### Step 3 - Act Tool Handler
1. Resolve tool alias to descriptor identity via turn-local alias map.
2. Parse arguments:
- `payload`
- `wait_for_sense`
3. Validate:
- descriptor exists in current turn map
- payload schema match
- `wait_for_sense <= max_waiting_seconds`
4. Materialize `Act` (`act_instance_id` generation as current path).
5. Append `EmittedAct` to cycle accumulator with `expected_fq_sense_ids`.

### Step 4 - Merge Expand Tools
1. Remove split tools:
- `expand-sense-raw`
- `expand-sense-with-sub-agent`
2. Add unified `expand-senses` in `primary.rs`.
3. Input:
- `mode`
- `senses_to_expand[]` (`sense_id`, optional `instruction`)
4. Dispatch:
- raw mode uses `sense_id` only.
- sub-agent mode requires `instruction` per item.
5. `sense_id` format:
- `"{monotonic_internal_sense_id}. {fq-sense-id}"`.

### Step 5 - Add Rule-Control and Sleep Tool Wrappers
1. Register static tools:
- `overwrite-sense-deferral-rule`
- `reset-sense-deferral-rules`
- `sleep`
2. Tool handlers:
- rule-control tools call afferent control port directly.
- `sleep` emits runtime control directive (`ignore_all_trigger_for_seconds`).

### Step 6 - Remove Legacy Text Act/Wait Path
1. Stop parsing acts/wait from output text:
- update `core/src/cortex/ir.rs` and `core/src/cortex/primary.rs`.
2. Remove `acts_output_helper` from active path:
- update `core/src/cortex/helpers/mod.rs`.
- remove/retire `core/src/cortex/helpers/acts_output_helper.rs`.
3. Update `core/src/cortex/prompts.rs`:
- remove `<somatic-acts>` and `<is-wait-for-sense>` output requirements.

### Step 7 - Runtime Wait Execution via Afferent Deferral
1. Update `core/src/cortex/runtime.rs` to consume `emitted_acts` in emission order.
2. For each emitted act:
- enqueue act to efferent producer.
- if `wait_for_sense_seconds == 0`, continue.
- else perform bounded wait window.
3. Wait window implementation:
- overwrite reserved afferent rule `__cortex_wait_non_target__` with non-target fq id regex.
- wait on afferent consumer for matching sense.
- clear rule by overwrite to no-match pattern `^$` on completion/timeout.
4. Match policy:
- strong match: `sense.act_instance_id == act.act_instance_id`.
- fallback: fq sense id in descriptor-declared emitted ids.

### Step 8 - Runtime Control and DI Wiring
1. Inject `Arc<dyn AfferentRuleControlPort>` where required by Primary/runtime.
2. Apply `output.control.ignore_all_trigger_for_seconds` in runtime:
- set `ignore_all_trigger_until = now + seconds`.
3. Ensure all control access is via DI from `main.rs` composition root (no global singleton access).

### Step 9 - Config and Schema Updates
1. Add `max_waiting_seconds` to:
- `core/src/config.rs`
- `core/beluna.schema.json`
2. Keep deterministic default and validation range.

## 3) File-Level Change Map
1. `core/src/cortex/types.rs`
2. `core/src/cortex/primary.rs`
3. `core/src/cortex/runtime.rs`
4. `core/src/cortex/ir.rs`
5. `core/src/cortex/prompts.rs`
6. `core/src/cortex/helpers/sense_input_helper.rs`
7. `core/src/cortex/helpers/mod.rs`
8. `core/src/cortex/helpers/acts_output_helper.rs` (remove/retire)
9. `core/src/config.rs`
10. `core/beluna.schema.json`
11. `core/src/main.rs`

## 4) Verification Gates
### Gate A - Legacy Path Removal
```bash
rg -n "somatic-acts|is-wait-for-sense|acts_output_helper|PRIMARY_TOOL_EXPAND_SENSE_RAW|PRIMARY_TOOL_EXPAND_SENSE_WITH_SUB_AGENT" core/src/cortex
```
Expected:
1. no active legacy act/wait text path.
2. no split expand tools.

### Gate B - Dynamic Act Tools
```bash
rg -n "wait_for_sense|dynamic act tool|tool_overrides|fq act id" core/src/cortex/primary.rs
```
Expected:
1. dynamic per-act tool registration exists.
2. each act tool accepts `payload` + `wait_for_sense`.

### Gate C - Expand-Senses Merge
```bash
rg -n "expand-senses|senses_to_expand|mode" core/src/cortex/primary.rs core/src/cortex/helpers/sense_input_helper.rs
```
Expected: merged tool path only.

### Gate D - Wait by Afferent Deferral
```bash
rg -n "__cortex_wait_non_target__|AfferentRuleControlPort|wait_for_sense_seconds" core/src/cortex
```
Expected:
1. wait window uses afferent rule-control port.
2. integer wait semantics in `[0, max_waiting_seconds]`.
3. `0` path is explicit no-wait.

### Gate E - Build
Per workspace rule:
```bash
cd core && cargo build
cd ../cli && cargo build
```

## 5) Completion Criteria (04)
1. Primary emits acts through dynamic per-act tools (transport-safe alias mapped to fq act ids).
2. `wait_for_sense` supports integer seconds with `0` no-wait behavior.
3. `expand-senses(mode)` with `senses_to_expand` replaces split tools.
4. Wait implementation is afferent-deferral based.
5. Core and CLI build successfully.

Status: `READY_FOR_REVIEW`
