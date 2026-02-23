# TODOs of Beluna

## Core

- [ ] 有 legder/ledger.rs，那为什么没有 cortex/cortex.rs 和 spine/spine.rs 呢
- [ ] 可不可以在 Spine, Cortex runtime 内实现 singleton 而不是 module 级别呢？
- [ ] config.rs 过耦合了其它业务，我认为就根据 json schema 来检查就可以了
- [ ] descriptor 缺少 description 字段 😆
- [ ] 文档化拓扑结构

### Cortex

- [ ] 让 Primary 不要给可选参数用默认值时传参，节省点 output token；或者往大了说就是不用太 deliberate
- [ ] 纠正 primary 的认知：Act 不是 tool，而是 willpower 落实到现实世界的 bridge
- [ ] 不是 wait-for-sense，而是 wait-for-act
- [ ] goal node 添加 status 字段 （注意更新 shot）

### Continuity

- [ ] 被动/主动回忆 与 被动/主动记忆；被动记忆还涉及到 sense 权重；Act其实不用记住，因为 Sense 会回传。

### Spine

- [ ] Spine Runtime 和 Body Endpoint Adapter 之间的交互给我搞清楚咯
- [ ] 测试应该在 tests/ 下面，有什么特殊的理由要 aside src 吗？
- [ ] 让 adapter 自己处理自己的 config

### Observability

- [ ] 拥抱 OpenTelemetry
- [ ] Local metrics (cortex-organ-output)
- [x] rotate，但是基于日期 + awake from hibernate times monotoic int

### AI Gateway

- [ ] attempt 是什么鬼
- [ ] 日志需要精简

### Std BodyEndpoint

- [ ] sense payload 需要优化，不要重复 metadata 中有的东西；
  比如 shell.result 的 payload 为什么要有 kind ?
  neural_signal_descriptor_id 更是诡异
- [ ] sense payload 不会携带 uuid 的 act-id ，要带也是带在 root，而且不可以 expose 给

#### Shell

## Apple Universal

- [x] 检查到 socket 存在不代表就要连接，把 Beluna 的状态和连接状态分开。
- [ ] 作为 Body Endpoint 哪来的 Spine ? 请直接命名为 BodyEndpoint 即可
- [x] Consolidate core's o11y into chat view:
  - 移动 metrics 到顶部，和状态
  - 将关键日志渲染为 tool call message
  - polling 日志或者说有更优雅的 watch
- [ ] Sense, Act persistence
