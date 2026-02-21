# TODOs of Beluna

## Core

- [x] 把 beluan core 的配置文件合并进入 beluna runtime, beluna core 只是一个 crate，它没有 main
- [x] Cortex 为核心，Stem 为枢纽；Organs (Spine, Continuity, Ledger, Motor) 为外围的有机结构
- [x] 移除所有的 TelemetrySink port, eprintln 等，全部改用 tracing，日志写入本地文件（采用 json log；rotate）
- [x] core/src/body 就是 std body 了，不用再包一层
- [x] ingress 破坏了 Beluna 的生物学隐喻式命名，我建议命名为 afferent pathway
- [x] ingress 应该包含创建 mpsc queue 的部分，而不是让 main 来创建
- [ ] 有 legder/ledger.rs，那为什么没有 cortex/cortex.rs 和 spine/spine.rs 呢
- [ ] 可不可以在 Spine, Cortex runtime 内实现 singleton 而不是 module 级别呢？
- [ ] config.rs 过耦合了其它业务，我认为就根据 json schema 来检查就可以了
- [ ] Stem Loop 按时间运行；cortex 可以触发 act sleep （注意区分休眠和睡眠）

### Cortex

- [x] Cortex contracts 中的 Act, Sense, Capability 移动到 types 中
- [ ] Cortex 的实现需要简化，目前搞得好混乱，不好调试还很慢
- [ ] Cognition State 还包含 context （但这是当前实现特定的，就作为一个字段就可以了）
- [ ] CortexCollaborators 是什么，和 AI Gateway 强耦合是预期行为
- [x] Cortex Config 来配置用什么 ai-provider-string 为 Primary, Serialize, Deserialize 等等
- [ ] 可以输出 cognition state，但是要注意不是整个栈都可以操作的，continuity可能会拒绝一些变化
- [ ] 我注意到给 input helper 的 neural signal descriptor 中有很多的转义符号，这很糟糕；type: act 应该过滤掉。
- [ ] 不要在 sense 中包含 uuid 等非语义性的内容，减少非语义噪音
- [ ] input helper 输出的是 json ，不是 output ir ... 让程序来组装数据，而不是 LLM 来组装
- [ ] text helper 是什么鬼，这是对 helper 的错误理解
- [ ] Metrics: cycle id, neural-signal-descriptor catalog, token consumed
- [ ] llm input / output log 是什么鬼，不应该让 ai gateway 来吗

### Spine

- [x] body 使用 pathway 是不可能的，它只能和 Spine 交互（更具体地说是 BodyEndpoint Adapter）
- [x] Inline Body Endpoint 和 Inline BodyEndpoint Adapter 之间的交互也要重新设计
- [ ] Spine Runtime 和 Body Endpoint Adapter 之间的交互给我搞清楚咯
- [ ] adapters/catalog_bridge 是什么鬼
- [x] 移除 body_endpoint_id ，name就是 id
- [x] 为什么要在 Spine runtime 中维护 adapter channel?
- [ ] 测试应该在 tests/ 下面，有什么特殊的理由要 aside src 吗？
- [x] new spine 不代表马上就要 start 啊
- [ ] 让 adapter 自己处理自己的 config

### Observability

- [ ] 拥抱 OpenTelemetry
- [x] Request ID
- [x] O11y in Error Handling
- [ ] Pull Metrics Endpoint

### AI Gateway

- [x] AI Gateway 重构
  - route by `backend-id/model-id`; can define a set of alias (eg. `default`, `low-cost`).
  - 配置文件要基于 backend 为首的结构
  - 提供能力特定的接口，而不是 infer_once, infer_stream 这样通用的接口。对于 result Ok 可以没有通用定义，但是 result Err 可以有。
- [ ] 移除 Credential Provider
- [ ] 啥是 attempt ?

## Apple Universal

- [x] 系统消息移到中间，而不是假装为 Beluna 说的
- [x] 将连接配置放到 SettingView 中
- [ ] 重连是指数退避的，最多重试5次；可以手动重试
- [ ] 检查到 socket 存在不代表就要连接，把 Beluna 的状态和连接状态分开。
- [ ] 哪来的那么多命令行窗口？
- [ ] 作为 Body Endpoint 哪来的 Spine ? 请命名为 BodyEndpoint
