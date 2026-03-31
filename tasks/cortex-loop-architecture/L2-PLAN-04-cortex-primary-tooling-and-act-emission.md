# L2 Plan 04 - Cortex Primary Tooling and Act Emission
- Task: `cortex-loop-architecture`
- Micro-task: `04-cortex-primary-tooling-and-act-emission`
- Stage: `L2`
- Date: `2026-03-01`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Goal and Scope
Goal:
1. Make act emission tool-call-native in Cortex Primary (remove text-parsed act output path).
2. Replace boolean wait with bounded per-act seconds (`wait_for_sense`, integer, `0` means no wait).
3. Expose each act as a dedicated tool with transport-safe alias mapped to fq act id (`endpoint-id/neural-signal-descriptor-id`).
4. Merge sense expansion tools into one `expand-senses(mode)` surface.
5. Implement `wait_for_sense` through afferent-pathway deferral control, not a Cortex wait-specific pending-sense buffer.

In scope:
1. Primary tool schemas and runtime execution contracts.
2. Cortex output shape changes required by tool-based act emission.
3. Runtime wait handling with bounded timeout.
4. Removal of legacy acts parsing/helper path.

Out of scope:
1. Goal forest reset transaction (`05`).
2. L1 memory removal and full cognition-shape simplification (`06`).
3. Efferent serial pipeline redesign (`07`).

## 2) Tool Surface (Primary Internal Tools)
Static tools:
1. `expand-senses`
2. `overwrite-sense-deferral-rule`
3. `reset-sense-deferral-rules`
4. `sleep`
5. `patch-goal-forest` (existing; `reset` extension remains in `05`)

Dynamic tools:
1. one act tool per available act descriptor.
2. tool name is transport-safe alias (for example `act_0001`) mapped to fq act id.
3. tool schema is descriptor-specific and generated per turn from current physical-state act descriptors.

### 2.1 Dynamic act tool schema template
For descriptor `D`:
```json
{
  "type": "object",
  "properties": {
    "payload": "<D.payload_schema>",
    "wait_for_sense": { "type": "integer", "minimum": 0 }
  },
  "required": ["payload", "wait_for_sense"],
  "additionalProperties": false
}
```

Validation rules:
1. tool alias must map to a known act descriptor in this turn.
2. `wait_for_sense <= limits.max_waiting_seconds`.
3. `wait_for_sense = 0` means no wait.
4. runtime still validates payload before materializing `Act`.

### 2.2 `expand-senses` schema (merged)
```json
{
  "type": "object",
  "properties": {
    "mode": { "type": "string", "enum": ["raw", "sub-agent"] },
    "senses_to_expand": {
      "type": "array",
      "minItems": 1,
      "items": {
        "type": "object",
        "properties": {
          "sense_id": { "type": "string", "minLength": 1 },
          "instruction": { "type": "string", "minLength": 1 }
        },
        "required": ["sense_id"],
        "additionalProperties": false
      }
    }
  },
  "required": ["mode", "senses_to_expand"],
  "additionalProperties": false
}
```

Mode routing:
1. `mode=raw`:
- each item uses only `sense_id`.
- `instruction` is ignored if provided.
2. `mode=sub-agent`:
- each item requires non-empty `instruction`.

`sense_id` contract:
1. format is `"{monotonic_internal_sense_id}. {fq-sense-id}"`.
2. the same value appears in Primary sense lines and tool arguments.

### 2.3 Afferent rule-control tool wrappers
`overwrite-sense-deferral-rule`:
1. maps directly to `AfferentRuleControlPort::overwrite_rule`.
2. overwrites one rule by `rule_id`.

`reset-sense-deferral-rules`:
1. maps directly to `AfferentRuleControlPort::reset_rules`.

### 2.4 `sleep` tool
Schema:
1. `seconds: integer` with `1 <= seconds <= max_waiting_seconds`.

Effect:
1. sets `ignore_all_trigger_until = now + seconds` in Cortex runtime through output control directive.

## 3) Runtime/Type Contracts (Frozen for L3)
Add explicit emitted-act envelope:

```rust
pub struct EmittedAct {
    pub act: Act,
    pub wait_for_sense_seconds: u64,
    pub expected_fq_sense_ids: Vec<String>,
}

pub struct CortexControlDirective {
    pub ignore_all_trigger_for_seconds: Option<u64>,
}

pub struct CortexOutput {
    pub emitted_acts: Vec<EmittedAct>,
    pub new_cognition_state: CognitionState,
    pub control: CortexControlDirective,
}
```

`expected_fq_sense_ids` source:
1. from optional act descriptor declaration (`emitted_sense_ids[]`).
2. empty list is allowed.

Naming policy:
1. external tool field is `wait_for_sense`.
2. internal runtime/type field is `wait_for_sense_seconds` for readability and bool/int deconfliction.

## 4) Wait Semantics via Afferent Deferral
Execution order is per emitted act (in emission order):
1. enqueue act to efferent producer.
2. if `wait_for_sense_seconds == 0`: continue.
3. otherwise run a bounded wait window.

Wait window algorithm:
1. compute expected fq sense ids for this act (`expected_fq_sense_ids`).
2. compute non-target fq sense ids from current known sense descriptor set.
3. build deferral regex for non-target ids and overwrite reserved rule id:
- `rule_id="__cortex_wait_non_target__"`
4. block on afferent consumer for matching sense until timeout:
- match by `sense.act_instance_id == act.act_instance_id`, else fallback fq id match.
5. on completion/timeout overwrite reserved rule to deterministic no-match (`fq_sense_id_pattern="^$"`).

Constraints:
1. no infinite wait.
2. wait implementation does not rely on a dedicated Cortex wait pending queue.
3. if non-target senses still leak through due unknown fq ids, they follow normal loop intake behavior after wait (no drop).

## 5) Primary Micro-Loop Contract Changes
1. Primary still runs on persistent AI Gateway chat thread (cycle is a thread turn).
2. Per turn, Primary receives:
- static tools
- dynamic act tools generated from current act descriptor set.
3. Act tool calls accumulate `EmittedAct` directly in runtime state.
4. Primary text output is no longer parsed for `<somatic-acts>` or wait flag.
5. Text output still supports currently retained cognition text sections (for example `new-focal-awareness` until `06`).

Sense text line contract:
```text
- [monotonic internal sense id]. [fq-sense-id]: [key=value,key=value,...]; [payload-truncated-if-needed]
```

## 6) Prompt/IR/Helper Deletion and Simplification
1. Remove `<somatic-acts>` and `<is-wait-for-sense>` output contract from Primary system prompt.
2. Remove `acts_output_helper` from active path.
3. Remove output-IR parsing dependency for acts/wait fields.
4. Keep `patch-goal-forest` behavior unchanged in this micro-task.

## 7) File/Interface Freeze for L3
1. `core/src/cortex/primary.rs`
2. `core/src/cortex/runtime.rs`
3. `core/src/cortex/types.rs`
4. `core/src/cortex/ir.rs`
5. `core/src/cortex/prompts.rs`
6. `core/src/cortex/helpers/sense_input_helper.rs`
7. `core/src/cortex/helpers/mod.rs`
8. `core/src/cortex/helpers/acts_output_helper.rs` (retire/remove)
9. `core/src/config.rs`
10. `core/beluna.schema.json`
11. `core/src/main.rs` (DI wiring for added control ports)

## 8) Risks and Constraints
1. Dynamic per-act tool fanout can increase tool payload size and latency.
Mitigation: deterministic per-turn tool generation and strict descriptor projection.
2. Fq act id as tool name may violate backend-specific naming constraints (for example slash restrictions).
Mitigation: use transport-safe alias tool names with deterministic fq id mapping.
3. Wait gates can interfere with user-authored afferent rules.
Mitigation: reserve namespace `__cortex_*`, deterministic overwrite-only lifecycle, explicit logs.

## 9) L2 Exit Criteria (04)
1. Dynamic per-act tool contract is frozen (transport-safe alias + fq mapping + payload + `wait_for_sense`).
2. `expand-senses(mode)` contract is frozen with `senses_to_expand`.
3. `wait_for_sense` integer semantics are frozen with `0` no-wait behavior.
4. Legacy acts parsing/helper path removal is frozen.
5. Wait algorithm is anchored to afferent deferral control.

Status: `READY_FOR_REVIEW`
