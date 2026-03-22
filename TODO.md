# TODOs of Beluna

- [ ] 迁移到 Github Issues & Project

## Core

- [x] 文档化拓扑结构
- [ ] descriptor 缺少 description 字段 😆
- [x] config.rs 过耦合了其它业务，我认为就根据 json schema 来检查就可以了 <https://gemini.google.com/u/1/app/19d2716163455423>
- [ ] 文档模式再重构 <https://chatgpt.com/share/69a94971-bebc-800c-a38e-b243c67d0efe>
- [x] 让 json schema 通过 cli 生成

### Cortex

- [ ] IR 改进 <https://gemini.google.com/u/1/app/e9c1e8ff7b2377bf>
- [x] 增加对 sense 消费的控制力 （权重；特定屏蔽）
- [x] 重构 goal tree
  - [x] weight 是基于一棵树的，而不是整个森林的
  - [x] 移除本能（加入到 Primary Prompt 中）
  - [x] 重新设计 node
  - [x] patch-goal-tool
- [ ] goal forest patching 总是不成功（改成 ops 就可以了但是），更多见 scratch/goal-forest-patch-issue-diagnosis.txt
  - [x] 解决方案尝试：将 goal-forest-patch 变为 natural-language based；
        结论：还是不太稳定，但至少可以了，随着后面 Dual Loop 的修复，删掉对 focal-awareness, somatic-acts 输出的要求等，使 content 就是纯粹的思考
- [x] Cortex / Stem Dual loop
  - [x] 除了 cognition-state，其它都不可以给 input-helper 主动"解释"
  - [x] 移除 l1-memory：goal-forest, 对话历史承担了这个角色
  - [x] 将 act 的输出也变为工具调用，但是立即返回；如果开启 wait，则等待响应的 sense (act-descriptor 配置 sense-matcher)
  - [x] goal-forest patch helper 应该自动生成 numbering 啊，numbering 是 required 的现在/
  - [x] goal-forest patch with reset
  - [x] efferent-pathway
  - [x] 驱动 primary turn: sense / tick.
  - [x] Add rule tool 没有捏？就最直接的 add / remove 吧，也不要什么 overwrite / reset 了。
  - [x] Sense internal monotonic id 需要基于进程声明周期而不是 cycle 周期
  - [x] Act dispatch failure as a tool result
  - [x] cognition-patch 这个模块很奇怪，应该删除；cognition 又是啥 ?
  - [x] cortex runtime 和 cortex 太割裂了 ... 但目前来看也没有必要 coupling
- [x] expand-sense 不成功
- [x] act tools 每次都被重建，导致了 tool-name 和 act 的对应关系因为正则化而不稳定；我建议继续维护 map，但是不要用 `act_000x` ，而是将 `/`, `.` 分别替换为 `_`, `-`；且对 Endpoint Id, NS Id 做约束。
- [x] only driven by tick
  - "Each admitted tick executes exactly one Cortex::cortex(...) " 不可接受
  - Continuation 似乎不是一个好的设计
- [x] sense input helper 还在干多余的事情
- [x] goal forest patch 丢东西
- [x] turn 0 goal forest patch with reset fails
  - add `trim_if_resolvable`
- [x] goal forest remains revision 1
- [ ] 进一步解除 cortex runtime 和 cortex 与 primary 的边界
- [ ] GoalForest 与 context reset 依然不工作
- [ ] Cortex 当前的设计存在系统性的问题，导致 "spinning in an idle “self-talk” loop with no new external input, generating high-volume repetitive logs."

### Continuity

- [ ] 被动/主动回忆 与 被动/主动记忆；被动记忆还涉及到 sense 权重；Act其实不用记住，因为 Sense 会回传。

### Spine

- [ ] Spine Runtime 和 Body Endpoint Adapter 之间的交互给我搞清楚咯
- [ ] 测试应该在 tests/ 下面，有什么特殊的理由要 aside src 吗？
- [ ] 让 adapter 自己处理自己的 config

### Observability

- [x] 拥抱 OpenTelemetry
- [ ] Local metrics (cortex-organ-output)
- [x] rotate，但是基于日期 + awake from hibernate times monotoic int
- [ ] Cortex 的日志设计还需要优化（比如`input_payload` 应该在 `body` 而不是 `attributes`；结构化而不是 string）

### AI Gateway

- [ ] attempt 是什么鬼
- [x] 日志需要精简
- [ ] Tool trait (mainly for o11y)
- [ ] retry, budget, relibability 都可以 consolidate 进入 adapter 实现

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
- [ ] 需要一个大重构，主要是 ChatView 且移除 core logs o11y
- [ ] send 的时候尝试连接一次，失败了就告诉 Beluna is disconnected
