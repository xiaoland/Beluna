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
- [x] Stem Loop 按时间运行；cortex 可以触发 act sleep （注意区分休眠和睡眠）
- [ ] descriptor 缺少 description 字段 😆
- [ ] 文档化拓扑结构

## Pathway

- [x] 区分 sense-id, act-id 于 sense-instance-id, act-instance-id

### Cortex

- [x] Cortex contracts 中的 Act, Sense, Capability 移动到 types 中
- [x] Cortex 的实现需要简化，目前搞得好混乱，不好调试还很慢
- [x] Cognition State 还包含 context （但这是当前实现特定的，就作为一个字段就可以了）
- [x] CortexCollaborators 是什么，和 AI Gateway 强耦合是预期行为
- [x] Cortex Config 来配置用什么 ai-provider-string 为 Primary, Serialize, Deserialize 等等
- [x] 可以输出 cognition state，但是要注意不是整个栈都可以操作的，continuity可能会拒绝一些变化
- [x] 我注意到给 input helper 的 neural signal descriptor 中有很多的转义符号，这很糟糕；type: act 应该过滤掉。
- [x] 不要在 sense 中包含 uuid 等非语义性的内容，减少非语义噪音
- [x] input helper 输出的是 json ，不是 output ir ... 让程序来组装数据，而不是 LLM 来组装
- [x] text helper 是什么鬼，这是对 helper 的错误理解
- [x] Metrics: cycle id, neural-signal-descriptor catalog, token consumed
- [x] llm input / output log 是什么鬼，不应该让 ai gateway 来吗
- [ ] Input IR 存在效率问题
  - [x] act-descriptor 存在 tag attrubutes 和 body markdown 重复的问题。
  - [x] 整个 Input IR act-descriptor 就应该是 markdown ，并且避免使用各种 text style markup。
- [x] Primary LLM 不是 transform , sir... 所以 Primary 的 LLM Prompt 应该是什么
- [x] InputIR GoalTree 现在什么情况，感觉很混乱
- [x] Cognition Organ 的 system prompt 和 user prompt 位置不对。user prompt 就是数据；system prompt 纯粹 instrutction
- [x] sense is sense, what is semantic sense ?
- [x] act-descripor helper 调用 LLM 来处理 payload schema 为 markdown，而不是整个 act descritor，其它字段比较meta，放在 XML 标签里面就很好
- [ ] 让 Primary 不要给可选参数用默认值时传参，节省点 output token；或者往大了说就是不用太 deliberate
- [x] willpower-matrix-patch 里面没有给出 numbering，要给个 one-shot 可能，或者说 user partition 一开始是空的时候给，后面它自然会有样学样；
- [x] focal-awareness 就是 bullet point statements
- [x] acts_helper 不需要把 output-ir 读进去；一方面会 duplicate，另一方面 act 解析不需要那么多上下文信息
- [x] primary 怎么不自言自语，感觉 prompt 还要调整
- [x] l1_memory_patch_helper 应该重命名为 l1-memory-flush helper 了
- [x] goal_tree_patch_helper 也不需要 output-ir
- [x] cortex 可以选择是否等待 sense arrived
- [x] goal-tree user_partition 怎么一直空空的，有bug
- [x] sense_helper 建议产出 payload 的 markdown，外面包一层 xml tag `<sense>` 以及 metadata 在 input-ir 中
- [x] goal-tree user-partition 才是 matrix-willpower
- [x] 什么鬼是 primary-helper ？需要重构，在代码模块层面理清楚 helper 与 primary
- [x] endpoint-id 和 neural-signal-descriptor-id 在 input ir 中合并为 (fully-qualified) act-id, sense-id；
      这意味着 output-ir 中输出的是 fully-qualified act-id，这时候就需要 helper 根据 catalog 拆开。
- [x] materialize 阶段遍历 catalog 拆 fq id 是不对的

### Continuity

- [ ] 被动/主动回忆 与 被动/主动记忆；被动记忆还涉及到 sense 权重；Act其实不用记住，因为 Sense 会回传。
- [x] 在给到Cortex Primary LLM的时候，重命名 l1-memory 为 scratchpad 或者别的生物学隐喻的东西
  - l1-memory -> Focol-Awareness
  - goal-tree -> Willpower-Matrix，其中 root-partition -> Instincts, User Partition -> Pursuits.
- [x] l1-memory 是 flush 而不是 patch (new-l1-memory)，且限制数量为 10 （可配置），超出的会被丢弃并且不告知（有日志）

### Spine

- [x] body 使用 pathway 是不可能的，它只能和 Spine 交互（更具体地说是 BodyEndpoint Adapter）
- [x] Inline Body Endpoint 和 Inline BodyEndpoint Adapter 之间的交互也要重新设计
- [ ] Spine Runtime 和 Body Endpoint Adapter 之间的交互给我搞清楚咯
- [x] adapters/catalog_bridge 是什么鬼，移除啊
- [x] 移除 body_endpoint_id ，name就是 id
- [x] 为什么要在 Spine runtime 中维护 adapter channel?
- [ ] 测试应该在 tests/ 下面，有什么特殊的理由要 aside src 吗？
- [x] new spine 不代表马上就要 start 啊
- [ ] 让 adapter 自己处理自己的 config

### Observability

- [ ] 拥抱 OpenTelemetry
- [x] Request ID
- [x] O11y in Error Handling
- [x] Pull Metrics Endpoint
- [ ] Local metrics (cortex-organ-output)
- [ ] rotate，但是基于日期 + awake from hibernate times monotoic int

### AI Gateway

- [x] AI Gateway 重构
  - route by `backend-id/model-id`; can define a set of alias (eg. `default`, `low-cost`).
  - 配置文件要基于 backend 为首的结构
  - 提供能力特定的接口，而不是 infer_once, infer_stream 这样通用的接口。对于 result Ok 可以没有通用定义，但是 result Err 可以有。
- [x] 移除 Credential Provider
- [ ] attempt 是什么鬼
- [ ] 日志需要精简

### Std BodyEndpoint

- [ ] sense payload 需要优化，不要重复 metadata 中有的东西；
  比如 shell.result 的 payload 为什么要有 kind ?
  neural_signal_descriptor_id 更是诡异
- [ ] sense payload 不会携带 uuid 的 act-id ，要带也是带在 root，而且不可以 expose 给

#### Shell

## Apple Universal

- [x] 系统消息移到中间，而不是假装为 Beluna 说的
- [x] 将连接配置放到 SettingView 中
- [x] 重连是指数退避的，最多重试5次；可以手动重试
- [ ] 检查到 socket 存在不代表就要连接，把 Beluna 的状态和连接状态分开。
- [x] 哪来的那么多命令行窗口？
- [ ] 作为 Body Endpoint 哪来的 Spine ? 请直接命名为 BodyEndpoint 即可
- [ ] Consolidate core's o11y into chat view:
  - 移动 metrics 到顶部，和状态
  - 将关键日志渲染为 tool call message
  - polling 日志或者说有更优雅的 watch
- [ ] Sense, Act persistence
