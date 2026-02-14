# TODOs of Beluna

- [ ] AI Gateway should route request by `provider.model_name`, use `default` as a special name.
- [ ] 把 beluan core 的配置文件合并进入 beluna runtime, beluna core 只是一个 crate，它没有 main
- [ ] Cortex adapters makes no sense，和 AI Gateway 强耦合是预期的行为
- [ ] Cortex 为核心，Stem 为枢纽；Organs (Spine, Continuity, Ledger, Motor) 为外围的有机结构

## Apple Universal

- [ ] 系统消息移到中间，而不是假装为 Beluna 说的
- [ ] 将连接配置放到 SettingView 中
- [ ] 重连是指数退避的，最多重试5次；可以手动重试
- [ ] 检查到 socket 存在不代表就要连接，把 Beluna 的状态和连接状态分开。
