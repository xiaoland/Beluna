# TODOs of Beluna

## Core

- [x] 把 beluan core 的配置文件合并进入 beluna runtime, beluna core 只是一个 crate，它没有 main
- [ ] Cortex adapters makes no sense，和 AI Gateway 强耦合是预期的行为
- [x] Cortex 为核心，Stem 为枢纽；Organs (Spine, Continuity, Ledger, Motor) 为外围的有机结构
- [ ] 日志写入本地文件，采用 json log
- [ ] Cortex contracts 中的 Act, Sense, Capability 移动到 types 中
- [ ] Cortex Config 来配置用什么 ai-provider-string 为 Primary, Serialize, Deserialize 等等

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
