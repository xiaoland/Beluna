# Stem Topography & Sequence

## Topography

Stem owns:
1. pathway construction (`afferent`, `efferent`)
2. tick authority
3. physical-state write path.

Stem does not invoke Cortex.

## Component Topography

```text
Stem Tick Runtime (core/src/stem/runtime.rs)
  └─ emits TickGrant over bounded channel

Stem Physical State Store (core/src/stem/runtime.rs)
  ├─ canonical Arc<RwLock<PhysicalState>>
  └─ write surface: StemControlPort

Afferent Pathway (core/src/stem/afferent_pathway.rs)
  ├─ bounded ingress queue
  ├─ deferral scheduler
  ├─ rule control: overwrite/reset/snapshot
  └─ observe-only sidecar events

Efferent Pathway (core/src/stem/efferent_pathway.rs)
  ├─ bounded FIFO queue
  ├─ producer handle used by CortexRuntime
  └─ serial consumer pipeline: Continuity -> Spine
```

## Runtime Sequence

```mermaid
sequenceDiagram
    participant Main
    participant ST as StemTickRuntime
    participant CR as CortexRuntime
    participant EF as EfferentRuntime
    participant CT as Continuity
    participant SP as Spine

    Main->>ST: spawn
    Main->>CR: spawn
    Main->>EF: spawn

    ST-->>CR: TickGrant
    CR->>CR: execute cycle on admitted tick (senses buffered between ticks)
    CR->>EF: enqueue EfferentActEnvelope

    loop FIFO serial acts
        EF->>CT: on_act(act, DispatchContext)
        alt Continue
            EF->>SP: on_act_final(act)
        else Break
            EF->>EF: mark REJECTED
        end
        EF->>EF: emit DISPATCHING/terminal proprioception status
    end
```

## Shutdown Sequence

```mermaid
sequenceDiagram
    participant Main
    participant AP as AfferentPathway
    participant RT as Runtimes
    participant EF as EfferentRuntime

    Main->>AP: close_gate()
    Main->>RT: cancel shutdown token
    EF->>EF: enter drain mode with timeout
    alt queue drained before timeout
        EF->>EF: shutdown complete
    else timeout
        EF->>EF: drop remaining envelopes with warning
    end
```
