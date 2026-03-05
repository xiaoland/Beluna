# Cortex Topography

## Runtime Boundary

```text
cortex(senses, physical_state) -> CortexOutput
```

`CortexOutput`:

1. `control: CortexControlDirective`
2. `pending_primary_continuation: bool`

## Component Topography

```text
CortexRuntime (core/src/cortex/runtime/mod.rs)
  ├─ owns cycle execution (tick-triggered only)
  ├─ keeps local pending sense queue between ticks
  ├─ buffers sense arrivals only; senses do not trigger immediate cycle execution
  ├─ applies sleep gate as tick-count suppression
  ├─ snapshots physical state via PhysicalStateReadPort
  ├─ calls Cortex::cortex(...)
  └─ drains buffered senses into each admitted tick cycle

Cortex Primary (core/src/cortex/runtime/primary.rs)
  ├─ assembles input IR sections (sense/proprioception/goal-forest)
  ├─ runs exactly one AI Gateway thread turn per cortex cycle
  ├─ injects previous tick tool-result messages into the next tick turn
  ├─ handles tool calls (act tools + internal cognitive tools)
  ├─ emits acts through efferent producer and returns ActDispatchResult in tool data
  ├─ persists cognition state through Continuity (direct call)
  └─ stores continuation batch when assistant returns tool_calls
```

## Primary Tools

1. Dynamic dedicated act tools (transport-safe alias -> fq act id mapping).
2. `expand-senses`:
   - tool arguments are direct `tasks[]` (`type: array`, `minItems: 1`)
   - task format: `{"sense_id":"<monotonic-id>", "use_subagent_and_instruction_is":"<instruction optional>"}`.
   - one call may mix raw tasks (without instruction) and sub-agent tasks (with instruction).
3. `patch-goal-forest` (`reset_context` supported).
4. `add-sense-deferral-rule`.
5. `remove-sense-deferral-rule`.
6. `sleep`.

## Sense Render Contract

Senses delivered to Primary:

```text
- [monotonic internal sense id]. endpoint_id=[endpoint_id], sense_id=[sense_id], weight=[weight][, truncated_ratio=[0..1 if truncated]]; payload="[payload-truncated-if-needed]"
```

## File Map

```text
core/src/cortex/
├── mod.rs
├── runtime/
│   ├── mod.rs
│   └── primary.rs
├── ir.rs
├── prompts.rs
├── clamp.rs
├── error.rs
├── testing.rs
├── types.rs
└── helpers/
```
