# TODOs of Beluna

## Core

- [ ] 文档化拓扑结构
- [ ] descriptor 缺少 description 字段 😆
- [ ] config.rs 过耦合了其它业务，我认为就根据 json schema 来检查就可以了 <https://gemini.google.com/u/1/app/19d2716163455423>
- [ ] 有 legder/ledger.rs，那为什么没有 cortex/cortex.rs 和 spine/spine.rs 呢
- [ ] 可不可以在 Spine, Cortex runtime 内实现 singleton 而不是 module 级别呢？

### Cortex

- [ ] IR 改进 <https://gemini.google.com/u/1/app/e9c1e8ff7b2377bf>
- [ ] 增加对 sense 消费的控制力
- [ ] 除了 cognition-state，其它都不可以给 input-helper 主动"解释"
- [x] 重构 goal tree
  - [x] weight 是基于一棵树的，而不是整个森林的
  - [x] 移除本能（加入到 Primary Prompt 中）
  - [x] 重新设计 node
  - [x] patch-goal-tool
- [ ] goal forest patching 总是不成功（改成 ops 就可以了但是），更多见 scratch/goal-forest-patch-issue-diagnosis.txt
  - 解决方案尝试：将 goal-forest-patch 变为 natural-language based
- [ ] Cortex / Stem Dual loop
  - [ ] 移除 l1-memory：goal-forest, 对话历史承担了这个角色
  - [ ] goal-forest helper 是全局单例，实现为一个 trait

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
- [x] 日志需要精简
- [ ] Tool trait (mainly for o11y)

### Std BodyEndpoint

- [ ] sense payload 需要优化，不要重复 metadata 中有的东西；
  比如 shell.result 的 payload 为什么要有 kind ?
  neural_signal_descriptor_id 更是诡异
- [ ] sense payload 不会携带 uuid 的 act instance id, 带在 metadata

#### Shell

## Apple Universal

- [x] 检查到 socket 存在不代表就要连接，把 Beluna 的状态和连接状态分开。
- [x] Consolidate core's o11y into chat view:
  - 移动 metrics 到顶部，和状态
  - 将关键日志渲染为 tool call message
  - polling 日志或者说有更优雅的 watch
- [x] Sense, Act persistence
- [x] 重构 Body Endpoint 部分
  - [x] 作为 Body Endpoint 哪来的 Spine ? 请直接命名为 BodyEndpoint 即可
  - [x] 优化 sense, act
- [ ] 需要一个大重构，主要是 ChatView
- [ ] send 的时候尝试连接一次，失败了就告诉 Beluna is disconnected。
