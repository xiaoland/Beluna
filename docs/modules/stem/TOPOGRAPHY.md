# Stem Topography & Sequence

## Topography

Stem 是 Beluna 核心运行时循环，负责 tick 调度、sense 摄入、Cortex 调用和串行 act 分发。

### 组件拓扑

```
                     ┌─────────────────────────────────────────────────────────────┐
                     │                      Stem Runtime                           │
 Afferent Pathway    │                      (stem.rs: Stem)                        │
══════════════════►  │                                                             │
 sense_rx (MPSC)     │  ┌─────────────────────────────────────────────────────┐    │
                     │  │               Main Loop (run)                       │    │
                     │  │                                                     │    │
                     │  │  StemMode::Active                                   │    │
                     │  │    ├─ wait_for_sense=true:  block on sense_rx.recv  │    │
                     │  │    └─ wait_for_sense=false: tick.tick() then drain   │    │
                     │  │                                                     │    │
                     │  │  StemMode::SleepingUntil(deadline)                  │    │
                     │  │    └─ select! { sleep(remaining) | sense_rx.recv }  │    │
                     │  │                                                     │    │
                     │  │  ──► execute_cycle(senses) ◄──                      │    │
                     │  │        │                                            │    │
                     │  │        ▼                                            │    │
                     │  │  ┌──────────────────────────┐                       │    │
                     │  │  │ 1. Apply control senses  │                       │    │
                     │  │  │    - NewNeuralSignal*     │                       │    │
                     │  │  │    - DropNeuralSignal*    │                       │    │
                     │  │  │    - New/DropProprioception│                      │    │
                     │  │  │ 2. cycle_id++             │                       │    │
                     │  │  │ 3. Compose physical_state │                       │    │
                     │  │  │ 4. Snapshot cognition     │                       │    │
                     │  │  │ 5. Cortex invocation      │──────►  Cortex       │    │
                     │  │  │ 6. Persist cognition      │──────►  Continuity   │    │
                     │  │  │ 7. Dispatch acts          │──────►  Dispatch Wkr │    │
                     │  │  └──────────────────────────┘                       │    │
                     │  └─────────────────────────────────────────────────────┘    │
                     │                                                             │
                     │  ┌─────────────────────────────────────────────────────┐    │
                     │  │           Dispatch Worker (async task)               │    │
                     │  │                                                     │    │
                     │  │  dispatch_rx.recv() loop:                           │    │
                     │  │    ┌───────────────────────────┐                    │    │
                     │  │    │ emit propri "DISPATCHING" │                    │    │
                     │  │    │ Continuity.on_act()       │──► Continue/Break  │    │
                     │  │    │   Break → REJECTED        │                    │    │
                     │  │    │   Error → LOST            │                    │    │
                     │  │    │   Continue ↓              │                    │    │
                     │  │    │ Spine.on_act_final()      │──► ACK/REJ/LOST   │    │
                     │  │    │ emit propri terminal stat │                    │    │
                     │  │    │ retain/drop status keys   │                    │    │
                     │  │    └───────────────────────────┘                    │    │
                     │  └─────────────────────────────────────────────────────┘    │
                     │                                                             │
                     │  Built-in: core.control/sleep act → StemMode::SleepingUntil │
                     └─────────────────────────────────────────────────────────────┘
```

### 文件拓扑

```
stem.rs                     Stem struct + run loop + execute_cycle + dispatch worker
afferent_pathway.rs         SenseAfferentPathway (gated MPSC sender wrapper)
```

### 依赖关系

```
Stem
 ├──► Cortex              (cortex() 调用, Arc<Cortex>)
 ├──► Continuity           (cognition snapshot/persist + on_act, Arc<Mutex<ContinuityEngine>>)
 ├──► Spine                (on_act_final dispatch, Arc<Spine>)
 └──► Afferent Pathway     (emit proprioception status patches, clone)
```

### 状态拓扑

```
Stem owns:
  cycle_id: u64                              单调递增周期号
  main_startup_proprioception: BTreeMap      启动时收集的只读系统属性
  dynamic_proprioception: BTreeMap           运行时动态属性（sense-driven）
  dispatch_tx / dispatch_task                分发工作线程通道与句柄
```

### Capability 合并

```
compose_physical_state() 合并三个 catalog:
  ┌─ Spine catalog          (body endpoint capabilities)
  ├─ Continuity catalog     (continuity-owned capabilities)
  └─ Stem catalog           (core.control/sleep)
  ────────────────────
  = merged NeuralSignalDescriptorCatalog (versioned)
```

---

## Sequence Diagram

### 正常 tick-driven 周期

```mermaid
sequenceDiagram
    participant AP as Afferent Pathway
    participant Stem
    participant Continuity
    participant Cortex
    participant DW as Dispatch Worker
    participant Spine

    Note over Stem: StemMode::Active, wait_for_sense=false

    Stem->>Stem: tick.tick() (interval wait)
    Stem->>AP: drain_senses_nonblocking()
    AP-->>Stem: sense_batch (may be empty)

    Note over Stem: execute_cycle begins

    loop Control sense processing
        alt NewNeuralSignalDescriptors
            Stem->>Continuity: apply_neural_signal_descriptor_patch()
        else DropNeuralSignalDescriptors
            Stem->>Continuity: apply_neural_signal_descriptor_drop()
        else NewProprioceptions
            Stem->>Stem: apply_proprioception_patch()
        else DropProprioceptions
            Stem->>Stem: apply_proprioception_drop()
        end
    end

    Stem->>Stem: cycle_id += 1
    Stem->>Continuity: cognition_state_snapshot()
    Continuity-->>Stem: CognitionState

    Stem->>Stem: compose_physical_state(cycle_id)
    Note over Stem: merge Spine + Continuity + Stem catalogs

    Stem->>Cortex: cortex(domain_senses, physical_state, cognition_state)
    Cortex-->>Stem: CortexOutput { acts, new_cognition_state, wait_for_sense }

    Stem->>Continuity: persist_cognition_state(new_cognition_state)

    loop For each act in CortexOutput.acts
        alt act is core.control/sleep
            Stem->>Stem: set StemMode::SleepingUntil(deadline)
            Note over Stem: break act loop
        else normal act
            Stem->>DW: dispatch_tx.send(DispatchTask)
        end
    end

    Note over Stem: CycleOutcome { sleep_deadline, wait_for_sense }
```

### Act 分发序列

```mermaid
sequenceDiagram
    participant Stem
    participant DW as Dispatch Worker
    participant AP as Afferent Pathway
    participant Continuity
    participant Spine

    Stem->>DW: DispatchTask { act, cycle_id, seq_no }
    activate DW

    DW->>AP: emit propri "DISPATCHING"

    DW->>Continuity: on_act(act, context)
    alt Continue
        DW->>Spine: on_act_final(act)
        alt Acknowledged
            DW->>AP: emit propri "ACK"
        else Rejected
            DW->>AP: emit propri "REJECTED"
        else Lost / Error
            DW->>AP: emit propri "LOST"
        end
    else Break (Continuity rejected)
        DW->>AP: emit propri "REJECTED"
    else Error
        DW->>AP: emit propri "LOST"
    end

    Note over DW: retain terminal status key (max 128), drop oldest

    deactivate DW
```

### wait_for_sense 模式

```mermaid
sequenceDiagram
    participant AP as Afferent Pathway
    participant Stem

    Note over Stem: Previous cycle returned wait_for_sense=true

    Stem->>AP: sense_rx.recv() (blocking)
    AP-->>Stem: first_sense

    alt first_sense is Hibernate
        Stem->>Stem: break loop → shutdown
    else
        Stem->>AP: drain_senses_nonblocking()
        AP-->>Stem: remaining senses
        Stem->>Stem: execute_cycle(all senses)
        Stem->>Stem: tick.reset()
    end
```

### Sleep 模式

```mermaid
sequenceDiagram
    participant AP as Afferent Pathway
    participant Stem
    participant Cortex

    Note over Stem: Cortex emitted core.control/sleep act

    Stem->>Stem: StemMode::SleepingUntil(now + seconds)

    alt sleep deadline reached first
        Stem->>Stem: execute_cycle(empty senses)
    else sense arrives before deadline
        AP-->>Stem: sense
        alt Hibernate
            Stem->>Stem: break loop → shutdown
        else
            Stem->>Stem: execute_cycle(senses)
        end
    end
```

### 启动与关停

```mermaid
sequenceDiagram
    participant Main
    participant AP as Afferent Pathway
    participant Stem
    participant DW as Dispatch Worker

    Main->>Stem: Stem::new(...) + tokio::spawn(stem.run())
    Stem->>DW: start_dispatch_worker()

    Note over Stem: Main loop begins

    Main->>Main: wait SIGINT/SIGTERM

    Main->>AP: close_gate()
    Main->>AP: send_hibernate_blocking()
    AP-->>Stem: Hibernate sense

    Stem->>Stem: break main loop
    Stem->>DW: shutdown_dispatch_worker() (drop tx, join task)
    Stem-->>Main: run() returns Ok(())
```
