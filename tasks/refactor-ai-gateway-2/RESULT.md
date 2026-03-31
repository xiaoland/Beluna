# AI Gateway Chat 重構 RESULT（本輪）

> Updated: 2026-02-24

## 已完成

1. `chat_once` 改為 adapter 原生路徑（已落地）
- `BackendAdapter` 增 `invoke_once(...) -> BackendOnceResponse`。
- `AIGateway::chat_once` 改走 `dispatch_once`，不再以 `chat_stream` 聚合模擬。
- `BackendOnceResponse` 已加入：`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/types_chat.rs`。

2. provider-first adapter 拓撲（已落地）
- OpenAI Compatible：`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/adapters/openai_compatible/chat.rs`
- Ollama：`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/adapters/ollama/chat.rs`
- GitHub Copilot：`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/adapters/github_copilot/chat.rs`
- `adapters/mod.rs` 已改為 provider 註冊。

3. OO Chat API（session/thread/turn）已落碼
- 新模組：`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/chat/mod.rs`
- 新 API：
  - `AIGateway::chat()`
  - `ChatGateway::open_session(...)`
  - `ChatSessionHandle::open_thread(...)`
  - `ChatThreadHandle::turn_once(...)`
  - `ChatThreadHandle::state(...)`
- In-memory session/thread store 已落：`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/chat/session_store.rs`
  - 含 thread 歷史管理、context 裁剪、session TTL。

4. usage sites 已實際遷移（非文檔層）
- `/Users/lanzhijiang/Development/Beluna/core/src/cortex/runtime.rs` 已由 `gateway.chat_once(...)` 直呼，改為 handle 流程：
  - primary 微循環：`session/thread/turn_once`
  - 各 organ 請求：`session/thread/turn_once`
- 產線代碼中已無 `gateway.chat_once(...)` 直調點。

5. 配置語義改版（default_* 命名一致）
- `AIGatewayConfig` 新增 `chat`：`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/types.rs`
- 已採 `default_*` 命名：
  - `default_route`
  - `default_max_tool_rounds`
  - `default_max_turn_context_messages`
  - `default_session_ttl_seconds`
  - `default_turn_timeout_ms`
- schema 已同步：`/Users/lanzhijiang/Development/Beluna/core/beluna.schema.json`

6. 可觀測性：以減日志增可讀（先行落地）
- gateway 內 request/response summary 已降為 `DEBUG`。
- telemetry `request_completed/request_cancelled` 已降為 `DEBUG`。
- 新增 `chat_turn_summary` / `chat_turn_anomaly` / `chat_session_lifecycle`（INFO 最小集）。
- 新增 task/thread metrics：`/Users/lanzhijiang/Development/Beluna/core/src/observability/metrics.rs`
  - task: latency/failures/retries
  - thread: turns/tool_calls/tokens/failures/last_latency

## 驗證

1. `cargo check --manifest-path /Users/lanzhijiang/Development/Beluna/core/Cargo.toml`：通過。

## 尚餘（未竟）

1. gateway 級 `tool orchestration` 尚未升格為統一路徑；primary 內部工具循環仍在 `cortex/runtime.rs`。
2. `ChatThreadHandle::turn_stream` 介面已留，但狀態提交語義尚未實作。
