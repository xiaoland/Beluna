# TODOs of Beluna

## Core

- [ ] 有 legder/ledger.rs，那为什么没有 cortex/cortex.rs 和 spine/spine.rs 呢
- [ ] 可不可以在 Spine, Cortex runtime 内实现 singleton 而不是 module 级别呢？
- [ ] config.rs 过耦合了其它业务，我认为就根据 json schema 来检查就可以了
- [ ] descriptor 缺少 description 字段 😆
- [ ] 文档化拓扑结构

### Cortex

- [ ] 添加 proprioception
- [x] 纠正 primary 的认知：Act 不是 tool，而是 willpower 落实到现实世界的 bridge
- [ ] 移除 wait-for-sense
- [x] Cognitive Sovereignty 属于 Primary
  - [x] Primary 变为多轮对话（不要求 LLM 一下就推出 act，这样可以更好地“注意”），称为 Cognitive Micro-loop；
        通过 tool-call 提供的是 Internal Cognitive Action ，不同于 Act （是Somatic Act）；
        设置 max_internal_steps 来避免死循环，未来可能需要 Watchdog Timeout
  - [x] 为 Primary LLM 提供 `expand-sense-raw(sense_ids: [ID_A, ID_B])`, `expand-sense-with-sub-agent(tasks: [{sense_id: ID_A, instruction: "..."}])` 工具;
        这些工具本质上都是 helper 的包装
  - [x] Sense Helper 不再把 payload 变为 cognition-friendly text，而是 Postman Envelope，包含 brief, original_size_in_bytes, confidence_score, omitted_features；当 payload size 较小时（可配置），透传。
- [x] primary-facing string optimization
  - act, sense -> somatic act / sense
- [x] act-descriptor-helper: 仅对结构复杂的 payload schema 做 cognition-friendly 转换
- [x] node-id 和 numbering 重复了啊...，改为 numbering, weight, summary, content, status 吧 （记得更新 shot）
- [x] willpower-matrix 提供 patch shot，否则 primary 不知道可以修剪、调权重，每次都全量输出去了
- [x] new-focal-awareness 给的 shot 有问题，导致 primary 输出 list 了，还一行一条
- [x] act-helper: 如果 act 已经是 valid json (也符合payload schema)，就不用调用 LLM 转换了
- [ ] 除了 cognition-state，其它都不可以被动地 convert
- [ ] 重构 goal tree
  - [ ] weight 是基于一棵树的，而不是整个森林的

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
- [x] Consolidate core's o11y into chat view:
  - 移动 metrics 到顶部，和状态
  - 将关键日志渲染为 tool call message
  - polling 日志或者说有更优雅的 watch
- [x] Sense, Act persistence
- [x] 重构 Body Endpoint 部分
  - [x] 作为 Body Endpoint 哪来的 Spine ? 请直接命名为 BodyEndpoint 即可
  - [x] 优化 sense, act
- [ ] 需要一个大重构，主要是 ChatView
