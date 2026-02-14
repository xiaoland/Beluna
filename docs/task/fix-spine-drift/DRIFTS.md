# Drifts of current Spine implementation from the desired one

- Spine 现在的 types.rs 里包含了 CostVector, reserve_entry_id, cost_attribution_id。
  - 然而 Spine 只是一个“盲”的 I/O 管道，负责安全和连接。
- Spine 应该支持多种 Transport Adapters 并存。然而现在
  - main.rs 硬编码了 UnixSocket adapter
  - unix_socket.rs 深度耦合了 Remote endpoint broker state，使得添加其他 Transport 变得困难
- Body Endpoint 在连接时可以声明自己的 "Dialect" (方言)；Spine 的 Adapter 负责将不同的 Dialect（如 Protobuf, Cap'n Proto, 或简单的 JSON）转换为 Core 统一的 Internal Message。
  - 然而 wire.rs 强绑定了 NDJSON (Newline Delimited JSON)；所有连接必须说 NDJSON，且遵循严格的 schema。
- Adapter 是一个抽象层，像插件一样加载
  - 然而现在unix_socket.rs 似乎不仅仅是 Adapter，它还承担了 Broker (代理) 的角色（管理 routes-per-endpoint, connected endpoint channels）;这意味着“连接管理逻辑”和“传输协议逻辑”混合在一起了。
