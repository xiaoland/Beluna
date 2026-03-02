# Core Topography

Beluna Core runtime topology (current implementation).

## Process Topology

```text
main()
  ├─ load config + init tracing + metrics exporter
  ├─ build Stem-owned afferent pathway handles
  ├─ build Stem physical-state store (StemControlPort)
  ├─ build Spine runtime (wired with afferent ingress + StemControlPort)
  ├─ register inline body endpoints
  ├─ build Continuity engine
  ├─ build Cortex (AI Gateway + Continuity + afferent rule-control)
  ├─ spawn StemTickRuntime
  ├─ spawn EfferentRuntime
  └─ spawn CortexRuntime
```

## Runtime Data Flow

```text
[Body endpoints + Spine dispatch failures] -> Afferent Pathway -> CortexRuntime
StemTickRuntime -> TickGrant channel -> CortexRuntime
CortexRuntime -> Efferent Pathway -> Continuity(on_act) -> Spine(on_act_final)
Spine adapters/runtime -> StemControlPort (ns_descriptor/proprioception updates)
StemPhysicalStateStore snapshot -> CortexRuntime (per cycle)
```

## Ownership Summary

1. `main`:
- composition root and lifecycle/shutdown orchestration.
2. Stem:
- tick authority
- afferent/efferent pathway ownership
- physical-state write ownership.
3. CortexRuntime:
- cycle execution ownership
- afferent consumer ownership.
4. Continuity:
- cognition persistence + act gate.
5. Spine:
- endpoint routing, adapter lifecycle, dispatch-failure sense emission.

## Shutdown Summary

1. receive SIGINT/SIGTERM
2. close afferent ingress gate
3. cancel runtime tokens
4. wait for Stem/Cortex tasks
5. wait for bounded efferent drain
6. flush continuity + shutdown spine
