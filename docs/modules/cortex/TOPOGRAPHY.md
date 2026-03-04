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
  ├─ owns cycle execution (tick + sense hybrid triggers)
  ├─ keeps local pending sense queue
  ├─ prioritizes pending primary continuation over new sense delivery
  ├─ snapshots physical state via PhysicalStateReadPort
  ├─ calls Cortex::cortex(...)
  └─ applies control gate (ignore_all_trigger_until)

Cortex Primary (core/src/cortex/runtime/primary.rs)
  ├─ assembles input IR sections (sense/proprioception/goal-forest)
  ├─ runs exactly one AI Gateway thread turn per cortex cycle
  ├─ handles tool calls (act tools + internal cognitive tools)
  ├─ emits acts through efferent producer and returns ActDispatchResult in tool data
  ├─ persists cognition state through Continuity (direct call)
  └─ stores continuation batch when assistant returns tool_calls
```

## Primary Tools

1. Dynamic dedicated act tools (transport-safe alias -> fq act id mapping).
2. `expand-senses`:
   - `mode: raw | sub-agent`
   - `senses_to_expand[].sense_id` format: `"<monotonic-id>. <fq-sense-id>"`.
3. `patch-goal-forest` (`reset_context` supported).
4. `add-sense-deferral-rule`.
5. `remove-sense-deferral-rule`.
6. `sleep`.

## Sense Render Contract

Senses delivered to Primary:

```text
- [monotonic internal sense id]. [fq-sense-id]: [key=value,key=value,...]; [payload-truncated-if-needed]
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
