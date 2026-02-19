# TODOs of Beluna

## Core

- [x] 把 beluan core 的配置文件合并进入 beluna runtime, beluna core 只是一个 crate，它没有 main
- [x] Cortex 为核心，Stem 为枢纽；Organs (Spine, Continuity, Ledger, Motor) 为外围的有机结构
- [ ] 移除所有的 TelemetrySink port，直接用 tracing，而后日志写入本地文件（采用 json log）
- [x] core/src/body 就是 std body 了，不用再包一层
- [x] ingress 破坏了 Beluna 的生物学隐喻式命名，我建议命名为 afferent pathway
- [x] ingress 应该包含创建 mpsc queue 的部分，而不是让 main 来创建
- [ ] 有 legder/ledger.rs，那为什么没有 cortex/cortex.rs 和 spine/spine.rs 呢
- [ ] 可不可以在 Spine, Cortex runtime 内实现 singleton 而不是 module 级别呢？

### Cortex

- [ ] CortexCollaborators 是什么，和 AI Gateway 强耦合是预期行为
- [ ] Cortex contracts 中的 Act, Sense, Capability 移动到 types 中
- [ ] Cortex Config 来配置用什么 ai-provider-string 为 Primary, Serialize, Deserialize 等等
- [ ] Cognition State 还包含 messages （但这是当前实现特定的，就作为一个字段就可以了）

### Spine

- [x] body 使用 pathway 是不可能的，它只能和 Spine 交互（更具体地说是 BodyEndpoint Adapter）
- [x] Inline Body Endpoint 和 Inline BodyEndpoint Adapter 之间的交互也要重新设计
- [ ] Spine Runtime 和 Body Endpoint Adapter 之间的交互给我搞清楚咯
- [ ] adapters/catalog_bridge 是什么鬼
- [ ] 移除 body_endpoint_id ，name就是 id
- [ ] 为什么要在 Spine runtime 中维护 adapter channel?
- [ ] 测试应该在 tests/ 下面，有什么特殊的理由要 aside src 吗？
- [ ] new spine 不代表马上就要 start 啊

### AI Gateway

- [ ] AI Gateway 重构
  - route by `backend-id/model-id`; can define a set of alias (eg. `default`, `low-cost`).
  - 配置文件要基于 backend 为首的结构
  - 提供能力特定的接口，而不是 infer_once, infer_stream 这样通用的接口。对于 result Ok 可以没有通用定义，但是 result Err 可以有。

## Apple Universal

- [x] 系统消息移到中间，而不是假装为 Beluna 说的
- [x] 将连接配置放到 SettingView 中
- [ ] 重连是指数退避的，最多重试5次；可以手动重试
- [ ] 检查到 socket 存在不代表就要连接，把 Beluna 的状态和连接状态分开。
- [ ] 哪来的那么多命令行窗口？
- [ ] 作为 Body Endpoint 哪来的 Spine ? 请命名为 BodyEndpoint
