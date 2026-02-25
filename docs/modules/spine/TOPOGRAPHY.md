# Spine Topography & Sequence

## Topography

Spine 是执行底座，位于 Stem 与 Body Endpoints 之间，负责 act 路由分发和 body endpoint 生命周期管理。

### 组件拓扑

```
                    ┌──────────────────────────────────────────────────────────────────┐
                    │                        Spine Runtime                             │
                    │                    (spine/runtime.rs: Spine)                      │
                    │                                                                  │
                    │  ┌────────────────────────────────────────────────────────────┐   │
                    │  │                    Process Singleton                       │   │
                    │  │              (GLOBAL_SPINE: OnceLock<Arc<Spine>>)          │   │
                    │  └────────────────────────────────────────────────────────────┘   │
                    │                                                                  │
                    │  ┌─────────────────────┐    ┌──────────────────────────────┐     │
                    │  │   Routing State      │    │   Endpoint State             │     │
                    │  │   (RwLock)           │    │   (Mutex)                    │     │
                    │  │                     │    │                              │     │
                    │  │  by_endpoint:       │    │  by_id:                     │     │
                    │  │   endpoint_id →      │    │   endpoint_id →             │     │
                    │  │   {dispatch,         │    │   RegisteredBodyEndpoint    │     │
                    │  │    descriptors}      │    │                              │     │
                    │  │                     │    │  by_channel:                │     │
                    │  │  adapter_channels:  │    │   channel_id → {endpoint_ids}│     │
                    │  │   channel_id →      │    │                              │     │
                    │  │   mpsc::Sender<Act> │    │                              │     │
                    │  │                     │    │                              │     │
                    │  │  version: u64       │    │                              │     │
                    │  └─────────────────────┘    └──────────────────────────────┘     │
                    │                                                                  │
                    │  ┌───────────────────────────────────────────────────────────┐    │
                    │  │                      Adapters                             │    │
                    │  │                                                           │    │
                    │  │  ┌─────────────────────┐   ┌──────────────────────────┐   │    │
                    │  │  │  Inline Adapter      │   │  UnixSocket Adapter      │   │    │
                    │  │  │  (OnceLock, 唯一)    │   │  (tokio::spawn task)     │   │    │
                    │  │  │                     │   │                          │   │    │
                    │  │  │  In-process body    │   │  NDJSON over UDS:       │   │    │
                    │  │  │  endpoints:         │   │  auth → register        │   │    │
                    │  │  │  - shell            │   │  sense → ingress        │   │    │
                    │  │  │  - web              │   │  act → egress           │   │    │
                    │  │  │                     │   │  act_ack → ingress      │   │    │
                    │  │  └─────────────────────┘   └──────────────────────────┘   │    │
                    │  └───────────────────────────────────────────────────────────┘    │
                    │                                                                  │
                    │  ┌───────────────────────────────────────────────────────────┐    │
                    │  │                  Act Dispatch Pipeline                    │    │
                    │  │                                                           │    │
  act ───────────►  │  │  on_act_final(act)                                       │    │
                    │  │    │                                                      │    │
                    │  │    ▼                                                      │    │
                    │  │  dispatch_act(act)                                        │    │
                    │  │    ├─ resolve endpoint_id → EndpointDispatch              │    │
                    │  │    │                                                      │    │
                    │  │    ├─ Inline: endpoint.invoke(act)                        │    │
                    │  │    │   └─ → Acknowledged / Rejected                       │    │
                    │  │    │                                                      │    │
                    │  │    └─ Adapter: tx.send(act) → channel                     │    │
                    │  │        └─ → Acknowledged / Lost                           │    │
                    │  │                                                           │    │
                    │  │  on failure → emit_dispatch_failure_sense() ──► AP        │    │
                    │  └───────────────────────────────────────────────────────────┘    │
                    └──────────────────────────────────────────────────────────────────┘
                                                                          │
                                                                          ▼
                                                              Afferent Pathway
                                                          (dispatch failure senses)
```

### 文件拓扑

```
spine/
├── mod.rs              公共导出 + GLOBAL_SPINE singleton
├── runtime.rs          Spine struct, routing, dispatch, endpoint registry, adapter lifecycle
├── endpoint.rs         Endpoint trait + NativeFunctionEndpoint
├── error.rs            SpineError / SpineErrorKind
├── types.rs            ActDispatchResult, EndpointExecutionOutcome, SpineEvent, SpineExecutionMode
├── AGENTS.md
└── adapters/
    ├── mod.rs          re-exports
    ├── inline.rs       SpineInlineAdapter (in-process body endpoints)
    └── unix_socket.rs  UnixSocketAdapter (external body endpoints via UDS+NDJSON)
```

### 依赖关系

```
Spine
 ├──► Afferent Pathway     (emit dispatch failure senses, clone)
 ├──► Endpoint trait        (inline endpoints implement this)
 └──► Config               (SpineRuntimeConfig → adapter configs)

Spine is used by:
 ├── Stem (dispatch worker calls on_act_final)
 ├── Main (boot, register inline endpoints, shutdown)
 └── Adapters (internal: register/remove endpoints, publish capabilities)
```

### 路由模型

```
Act routing is endpoint_id-level only:

  act.endpoint_id  ──lookup──►  EndpointDispatch
                                  ├─ Inline(Arc<dyn Endpoint>)
                                  └─ AdapterChannel(channel_id)

Capability routing (neural_signal_descriptor_id) is delegated to endpoint internals.
```

### EndpointDispatch 结果

```
ActDispatchResult:
  ├─ Acknowledged { reference_id }     成功接收
  ├─ Rejected { reason_code, ref_id }  端点拒绝
  └─ Lost { reason_code, ref_id }      传输丢失
```

---

## Sequence Diagram

### Act 分发（完整路径）

```mermaid
sequenceDiagram
    participant DW as Stem Dispatch Worker
    participant Spine
    participant RT as Routing Table
    participant EP as Body Endpoint
    participant AP as Afferent Pathway

    DW->>Spine: on_act_final(act)
    activate Spine

    Spine->>Spine: dispatch_act(act)
    Spine->>RT: resolve_dispatch(act.endpoint_id)

    alt endpoint found (Inline)
        RT-->>Spine: EndpointDispatch::Inline(endpoint)
        Spine->>EP: endpoint.invoke(act)
        alt success
            EP-->>Spine: Acknowledged { reference_id }
        else invoke error
            EP-->>Spine: error
            Spine->>Spine: wrap as Rejected
        end
    else endpoint found (Adapter)
        RT-->>Spine: EndpointDispatch::AdapterChannel(channel_id)
        Spine->>RT: adapter_channels[channel_id].send(act)
        alt send ok
            RT-->>Spine: Acknowledged
        else channel closed
            RT-->>Spine: error
            Spine->>Spine: wrap as Lost
        end
    else endpoint not found
        RT-->>Spine: None
        Spine->>Spine: Rejected { endpoint_not_found }
    end

    alt Rejected or Lost
        Spine->>AP: emit_dispatch_failure_sense(act, reason, ref_id)
    end

    Spine-->>DW: ActDispatchResult
    deactivate Spine
```

### Body Endpoint 注册（inline）

```mermaid
sequenceDiagram
    participant Main
    participant IA as Inline Adapter
    participant Spine
    participant RS as Routing State
    participant AP as Afferent Pathway

    Main->>IA: register body endpoint(name, handler, descriptors)
    IA->>Spine: add_endpoint(name, Inline(endpoint), descriptors)
    activate Spine

    Spine->>Spine: allocate body_endpoint_id = "{name}.{seq}"
    loop for each descriptor
        Spine->>Spine: descriptor.endpoint_id = body_endpoint_id
        Spine->>RS: upsert_route(descriptor, Inline dispatch)
        RS->>RS: routing.version += 1
    end

    Spine->>Spine: register in endpoint_state.by_id
    Spine-->>IA: BodyEndpointHandle { body_endpoint_id }
    deactivate Spine

    IA->>AP: emit NewNeuralSignalDescriptors sense
    Note over AP: Stem 下一个 cycle 会将新 capability 合并到 physical_state
```

### Body Endpoint 注册（外部 UnixSocket）

```mermaid
sequenceDiagram
    participant Ext as External Body Endpoint
    participant UDS as UnixSocket Adapter
    participant Spine
    participant AP as Afferent Pathway

    Ext->>UDS: connect (Unix Domain Socket)
    UDS->>Spine: on_adapter_channel_open(adapter_id, tx)
    Spine-->>UDS: channel_id

    Ext->>UDS: auth { endpoint_name, descriptors }
    UDS->>Spine: add_endpoint(name, AdapterChannel(channel_id), descriptors)
    Spine-->>UDS: BodyEndpointHandle

    UDS->>AP: emit NewNeuralSignalDescriptors sense

    Note over UDS: Bidirectional NDJSON streaming begins

    loop act egress
        Spine->>UDS: adapter_channels[channel_id].send(act)
        UDS->>Ext: NDJSON act line
        Ext->>UDS: act_ack { result }
        UDS->>AP: sense (ack feedback)
    end

    loop sense ingress
        Ext->>UDS: sense { payload }
        UDS->>AP: Domain sense
    end
```

### Endpoint 断开与清理

```mermaid
sequenceDiagram
    participant UDS as UnixSocket Adapter
    participant Spine
    participant RS as Routing State
    participant AP as Afferent Pathway

    Note over UDS: connection lost / disconnect

    UDS->>Spine: on_adapter_channel_closed(channel_id)
    activate Spine

    Spine->>RS: remove adapter_channels[channel_id]
    Spine->>Spine: lookup endpoint_ids by channel_id

    loop for each endpoint_id
        Spine->>Spine: remove_endpoint(endpoint_id)
        Spine->>RS: remove all routes for endpoint
        RS->>RS: routing.version += 1
    end

    Spine-->>UDS: dropped route_keys[]
    deactivate Spine

    UDS->>AP: emit DropNeuralSignalDescriptors sense (dropped routes)
    Note over AP: Stem 下一个 cycle 会从 physical_state 中移除这些 capability
```

### Spine 启动与关停

```mermaid
sequenceDiagram
    participant Main
    participant Spine
    participant IA as Inline Adapter
    participant UDS as UnixSocket Adapter

    Main->>Spine: Spine::new(config, afferent_pathway)
    activate Spine
    Spine->>Spine: start_adapters(config)

    alt Inline adapter configured
        Spine->>IA: SpineInlineAdapter::new()
        Spine->>Spine: inline_adapter.set(adapter)
    end

    alt UnixSocket adapter configured
        Spine->>UDS: tokio::spawn(adapter.run())
    end

    Spine-->>Main: Arc<Spine>
    deactivate Spine

    Main->>Spine: install_global_spine()
    Main->>IA: register_inline_body_endpoints(shell, web)

    Note over Main: ... runtime runs ...

    Main->>Spine: shutdown()
    Spine->>Spine: shutdown.cancel()
    Spine->>UDS: await adapter tasks
    Spine-->>Main: Ok(())
```
