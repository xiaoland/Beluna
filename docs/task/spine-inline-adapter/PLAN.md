# Spine Inline Adapter Plan (v3)

## 1. Goal

定义 Inline Body Endpoint 与 Spine Inline Adapter 的交互模型，满足这些硬约束：

1. Spine Inline Adapter 不依赖 Inline Body Endpoints 的具体实现。
2. 每个 Inline Body Endpoint 运行在独立线程，由 `main` 按 `config.body` 启动。
3. Inline Body Endpoint 与 Spine Inline Adapter 通过进程内堆内存共享（MPSC）通信。
4. Spine Adapter 只做 Sense 归一化并提交给 Afferent Pathway。
5. Spine Runtime 只路由到 Adapter；Adapter 再路由到目标 endpoint。
6. `main` 不直接启动 Spine Adapters；Spine Runtime 按 `spineConfig.adapters` 启动。

## 2. Locked Decisions

1. 用语统一为 `Afferent Pathway`，不再使用 deprecated 的 `ingress`。
2. 统一命名为 `Adapter`，不使用 `AdapterChannel`。
3. Inline Adapter 不实现 ACK 机制：Act 成功写入 endpoint act mailbox 即视为 dispatch 成功并结束。
4. Inline Adapter 不使用 `ActDispatchAck` 通道；执行产出统一通过 Sense 回传。
5. sense 背压采用 MPSC 默认阻塞语义（不丢弃）。
6. 用于 inline adapter 通信的 mailbox 由 Spine Inline Adapter 创建并持有生命周期。
7. endpoint 启动只注入 inline adapter 实例；不采用 Hexagonalization pattern。

## 3. Responsibility Boundary

### Spine Runtime

1. 启动并持有 adapters（来自 `spineConfig.adapters`）。
2. 维护 `route -> adapter` 路由注册。
3. `dispatch_act(act)` 只把 Act 发送给目标 adapter。
4. 不感知 endpoint 线程、endpoint mailbox 等细节。

### Spine Inline Adapter

1. 持有 endpoint 注册表与 endpoint mailbox 生命周期。
2. 负责创建 mailbox，并把 endpoint-side handles 交给 endpoint 线程使用。
3. 接收 Runtime 的 Act 后按 endpoint name 路由并写入 endpoint mailbox。
4. 接收 endpoint Sense 后做最小归一化并提交给 Afferent Pathway。
5. 管理 endpoint 注册/注销、capability route 同步、线程退出清理。

### Inline Body Endpoint

1. 由 `main` 启动独立线程。
2. 启动时仅拿到 `Arc<SpineInlineAdapter>` 实例。
3. 在线程初始化时向 adapter attach 自己，获取 endpoint-side runtime handles。
4. 接收 Act 后直接执行，执行结果统一通过 Sense 回传。

## 4. Data Model (Min-Conversion)

目标：adapter 与 endpoint 间尽量复用 Spine 的 Act/Sense 数据结构，减少转换。

1. Act 通道传 `Act`（建议 `Arc<Act>`）。
2. Sense 通道传 `SenseDatum`（建议 `Arc<SenseDatum>`）。
3. Inline Adapter 不定义 endpoint ACK 数据模型。

说明：

1. Dispatch 成功条件是 adapter 成功写入 endpoint act mailbox。
2. Dispatch 一旦完成写入即 End；执行产出继续走 Sense pipeline。

## 5. Adapter-Owned Mailbox Model

关键原则：mailbox 由 inline adapter 创建和持有，角色上等价于 Unix Socket Adapter 持有 socket。

### 5.1 Attach API (Concrete Type, No Hexagonalization)

```rust
impl SpineInlineAdapter {
    pub fn attach_inline_endpoint(
        self: &Arc<Self>,
        endpoint_name: String,
        capabilities: Vec<EndpointCapabilityDescriptor>,
    ) -> anyhow::Result<InlineEndpointRuntimeHandles>;
}

pub struct InlineEndpointRuntimeHandles {
    pub act_rx: tokio::sync::mpsc::Receiver<Arc<Act>>,
    pub sense_tx: tokio::sync::mpsc::Sender<Arc<SenseDatum>>,
}
```

### 5.2 Ownership

1. Adapter 持有：
   - endpoint registry
   - adapter-side senders/receivers
   - 路由与清理状态
2. Endpoint 线程只持有运行所需 endpoint-side handles（来自 attach 返回值）。

## 6. Startup Flow

1. `main` 读取 config。
2. `main` 创建 Spine Runtime。
3. Spine Runtime 按 `spineConfig.adapters` 启动 adapters（含 inline adapter）。
4. `main` 从 Runtime 获取 inline adapter 实例。
5. `main` 按 `config.body.*` 启动每个 endpoint 线程，并把 inline adapter 实例传入启动函数。
6. endpoint 线程调用 `inline_adapter.attach_inline_endpoint(...)`。
7. attach/register 失败即启动失败，`main` 直接中断启动流程。
8. `main` 启动 Stem（不做额外“路由就绪检查”）。

## 7. Dispatch Flows

### 7.1 Act -> Enqueue -> End

1. Stem -> Spine Runtime: `dispatch_act(act)`。
2. Runtime -> Adapter: 发送 Act。
3. Adapter -> Endpoint: 写入 endpoint `act_rx` 对应 mailbox。
4. 写入成功则 dispatch 结束（End）；写入失败则按拒绝语义返回。

### 7.2 Sense

1. Endpoint -> Adapter: 发送 `SenseDatum`。
2. Adapter: 最小归一化（仅补必要字段，不做业务解释）。
3. Adapter -> Afferent Pathway: 提交 `Sense::Domain(...)`。

## 8. Failure Semantics

1. endpoint 未注册：dispatch 失败（`endpoint_not_found`）。
2. endpoint mailbox 已断开：dispatch 失败（`endpoint_unavailable`）。
3. endpoint 线程退出：adapter 自动注销路由并 drop capabilities。
4. sense 发送采用阻塞背压；队列满时等待，不丢弃。
5. endpoint attach/register 失败：核心启动失败。

## 9. Implementation Stages

### Stage A

1. 在 Spine contracts 中定义 inline dispatch 语义：enqueue success => dispatch success（无 endpoint ACK）。
2. 统一文档和代码术语为 Afferent Pathway。

### Stage B

1. Runtime 实现严格的 `route -> adapter` 路由语义。
2. Inline adapter 实现 adapter-owned mailbox + attach API + enqueue 流程。

### Stage C

1. `main` 改为只注入 inline adapter 实例给 endpoint 启动函数。
2. `core/src/body/*` 迁移到 attach 模型（endpoint 线程内自注册）。

### Stage D

1. 增加行为测试：Runtime 只到 adapter，adapter 再到 endpoint。
2. 增加故障测试：enqueue 失败、endpoint 退出、register 失败、sense 背压阻塞。

## 10. Open Item

1. 是否需要为 inline adapter 增加 dispatch enqueue timeout 配置（如 `spine.adapters.inline.enqueue_timeout_ms`）？
