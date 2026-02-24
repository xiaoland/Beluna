# AI Gateway Usage Sites（2026-02-24）

> 旨：列明誰直接調用 `ai_gateway`，誰間接依賴，及其調用形態。

## 1. 直調點（Production）

1. `/Users/lanzhijiang/Development/Beluna/core/src/main.rs`
- 建構單例 `AIGateway::new(config.ai_gateway, EnvCredentialProvider)`。
- 將 gateway 注入 `Cortex`。

2. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/runtime.rs`
- 已改為物件化 Chat 入口：`gateway.clone().chat()`。
- primary 與 organ 皆改走：
  - `open_session(...)`
  - `open_thread(...)`
  - `turn_once(...)`
- 產線 usage site 已無 `gateway.chat_once(...)` 直呼。

## 2. 間接依賴點（Production）

下列 helper 不直持 gateway，惟經 `runtime.run_organ -> chat thread turn_once` 間接觸發模型調用：

1. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/helpers/sense_input_helper.rs`
2. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/helpers/acts_output_helper.rs`
3. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/helpers/goal_tree_patch_output_helper.rs`
4. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/helpers/l1_memory_flush_output_helper.rs`
5. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/helpers/act_descriptor_input_helper.rs`
6. `/Users/lanzhijiang/Development/Beluna/core/src/cortex/helpers/goal_tree_input_helper.rs`

## 3. Adapter 使用拓撲（Implementation）

1. 註冊總表：`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/adapters/mod.rs`
- 以 `BackendDialect -> Arc<dyn BackendAdapter>` 供 gateway 分派。

2. provider-first 模組：
- `/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/adapters/openai_compatible/chat.rs`
- `/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/adapters/ollama/chat.rs`
- `/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/adapters/github_copilot/chat.rs`

3. 現行調用語義：
- `chat_once` -> adapter `invoke_once`（原生 once）
- `chat_stream` -> adapter `invoke_stream`（串流）
- `ChatThreadHandle::turn_once` -> gateway `chat_once`（由 thread state 組上下文）

## 4. 測試使用點

1. `/Users/lanzhijiang/Development/Beluna/core/tests/ai_gateway/gateway_e2e.rs`
- 直接構造 `AIGateway` + mock adapter。
- 覆蓋 `chat_once` 與 `chat_stream`。

2. `/Users/lanzhijiang/Development/Beluna/core/tests/ai_gateway/openai_compatible.rs`
3. `/Users/lanzhijiang/Development/Beluna/core/tests/ai_gateway/ollama.rs`
4. `/Users/lanzhijiang/Development/Beluna/core/tests/ai_gateway/copilot_adapter.rs`
- 聚焦各 adapter 契約與錯誤映射。

5. `/Users/lanzhijiang/Development/Beluna/core/tests/ai_gateway/router.rs`
- 檢 alias 與 `backend/model` 直路由。

## 5. 現存差距（相對重構終局）

1. `tool orchestration` 仍主要在 `cortex/runtime.rs`（尚未下沉為 gateway 內建回合循環）。
2. `ChatThreadHandle::turn_stream` 已留介面，尚未實作狀態提交語義。
3. `cli/`、`apple-universal/` 目前未直接調 `ai_gateway`。
