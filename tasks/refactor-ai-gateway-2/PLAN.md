# AI Gateway Chat 重構 Low-level Design PLAN

> Last Updated: 2026-02-24
> 性質：執行藍圖（LLD）；以破舊立新為旨，不保 backward compatibility。

## 1. 重構主張（以今見對齊其制）

本輪以三義為綱：

1. `AI Gateway` 對外每一「門」即一獨立 AI 能力；能力為一級邊界。
2. `BackendAdapter` 之形，當與能力門同形；禁跨能力混雜與多重轉譯。
3. `Chat` 當名副其實：內含 session/thread、多輪上下文、工具編排（tool orchestration），而非僅單輪包裝。
4. `chat_once` 必由 adapter 原生實作；不得由 gateway 聚合 `chat_stream` 假擬而成。

## 2. 目標與非目標

### 2.1 目標

1. Local Refactoring：正名、抽函、去重、減少閱讀跳轉成本。
2. Structural Refactoring：按能力拆模組、重設邊界、改資料流、替換不當 pattern。
3. Architectural Reset：重寫 Chat 子系統，使 session/thread/tool orchestration 升格為核心能力。

### 2.2 非目標

1. 本輪不追求舊 API 相容。
2. 本輪不引入 multi-backend fallback（仍保 deterministic single-route）。
3. 本輪不同時擴展 ASR/TTS 等新能力門。

## 3. 目標拓撲（Post-Reset）

```text
core/src/ai_gateway/
├── mod.rs
├── gateway.rs                       # 能力門聚合入口（僅分發，不承載 Chat 細節）
├── shared/
│   ├── error.rs                     # GatewayError（跨能力）
│   ├── credentials.rs               # 憑證解決
│   ├── telemetry.rs                 # 通用遙測 façade
│   ├── budget.rs                    # 通用預算元件
│   └── reliability.rs               # 通用可靠性元件
├── chat/
│   ├── mod.rs
│   ├── api.rs                       # 對外 Chat 能力 API
│   ├── types.rs                     # ChatSession/Thread/Turn/Event/Tool 型別
│   ├── service.rs                   # ChatService（能力主協調者）
│   ├── session_store.rs             # Session/Thread 狀態存取邊界
│   ├── orchestrator.rs              # Tool orchestration + turn loop
│   ├── router.rs                    # Chat 專用 route 決策
│   ├── normalizer.rs                # Chat 請求規範化
│   ├── response.rs                  # Chat 回應規範化
│   ├── policy.rs                    # Chat 專屬限制（max_turns/tool_rounds 等）
│   └── backend_contract.rs          # trait ChatBackendAdapter + backend-facing DTO
└── adapters/
    ├── mod.rs                       # provider registry（provider-first）
    ├── openai_compatible/
    │   ├── mod.rs
    │   ├── chat.rs                  # impl ChatBackendAdapter for OpenAI-compatible
    │   └── http_common.rs
    ├── ollama/
    │   ├── mod.rs
    │   └── chat.rs                  # impl ChatBackendAdapter for Ollama
    └── github_copilot/
        ├── mod.rs
        ├── chat.rs                  # impl ChatBackendAdapter for Copilot SDK
        └── rpc.rs
```

設計斷語：`chat/*` 專注能力邊界；`shared/*` 僅容跨能力通用政策；`adapters/*` 採 provider-first，於 provider 內再以能力檔案（如 `chat.rs`）對齊 gateway 能力門，兼得可讀與可擴。

## 4. 新抽象（Low-level Contract）

### 4.1 對外 Chat API（OO 形態，破舊）

以物件樹暴露能力，避免函式平鋪使 gateway 表面膨脹：

1. `AIGateway::chat() -> Arc<ChatGateway>`
2. `ChatGateway::open_session(req) -> ChatSessionHandle`
3. `ChatSessionHandle::open_thread(req) -> ChatThreadHandle`
4. `ChatThreadHandle::turn_once(req) -> ChatTurnResponse`
5. `ChatThreadHandle::turn_stream(req) -> ChatTurnEventStream`
6. `ChatThreadHandle::state() -> ChatThreadState`
7. `ChatSessionHandle::close() -> ()`

關鍵變更：

- 會話與線程以 handle 類型承載，隱去重複傳參。
- `turn_*` 不再要求呼叫方全量提供歷史；gateway 自 thread state 組上下文。
- 舊 `ChatRequest.route/messages/...` 直傳模型之模式移除。

### 4.2 Chat 狀態模型

1. `ChatSession`：
- `session_id`
- `default_route`
- `created_at`, `expires_at`
- `policy`（context window、max_tool_rounds、max_turn_time）

2. `ChatThreadState`：
- `thread_id`
- `messages`（canonical）
- `next_turn_index`
- `pending_tool_calls`（若有）
- `summary_checkpoint`（可選，供長上下文截斷）

3. `ChatTurnRequest`：
- `session_id`, `thread_id`
- `input_parts`（本回合新訊息）
- `tools`（本回合可用工具集）
- `route_override`（可選）
- `limits_override`（可選）
- `metadata`, `cost_attribution_id`

### 4.3 能力同形 Adapter 契約

```rust
#[async_trait]
pub trait ChatBackendAdapter: Send + Sync {
    fn dialect(&self) -> ChatBackendDialect;
    fn capabilities(&self) -> ChatBackendCapabilities;

    async fn chat_turn_once(
        &self,
        ctx: ChatAdapterContext,
        req: ChatBackendRequest,
    ) -> Result<ChatBackendOnceResult, GatewayError>;

    async fn chat_turn_stream(
        &self,
        ctx: ChatAdapterContext,
        req: ChatBackendRequest,
    ) -> Result<ChatBackendStreamInvocation, GatewayError>;
}
```

規約：

1. `chat_turn_once` 與 `chat_turn_stream` 皆為必實作；不得在 gateway 層以一擬一。
2. 若某 provider 天生無 once 端點，轉譯責任在該 adapter 內部自決；不可上拋到 gateway。
3. adapter 輸出 `ChatBackend*` 型別，與 Chat 能力語義對齊，不得混入他能力欄位。
4. provider 模組分層採 `adapters/<provider>/<capability>.rs`，目錄按 adapter，實作按能力。

### 4.4 Tool Orchestration 升格

新增 `ChatToolRuntime` 邊界：

```rust
#[async_trait]
pub trait ChatToolRuntime: Send + Sync {
    async fn execute(
        &self,
        call: CanonicalToolCall,
        ctx: ChatToolExecutionContext,
    ) -> Result<ChatToolResult, GatewayError>;
}
```

`ChatService` 內建回合循環：

1. 取 thread 歷史 + 本回合輸入。
2. 呼叫 adapter（once 或 stream）。
3. 若有 tool call，交 `ChatToolRuntime` 執行並回填 tool message。
4. 續呼模型，至「無工具呼叫」或達 `max_tool_rounds`。
5. 原子寫回 thread 狀態，產出最終 turn 結果。

## 5. 資料流重設

### 5.1 `chat_turn_once`

1. `ChatService` 讀 session/thread。
2. `chat::normalizer` 生成 `ChatBackendRequest`。
3. `chat::router` 決 `(backend, model)`。
4. `shared::credentials/budget/reliability` 做前置守衛。
5. 調 `ChatBackendAdapter::chat_turn_once`。
6. 若含 tool call，交 `orchestrator` 迴圈；否則直接收斂。
7. 原子持久化 thread + telemetry 結算。

### 5.2 `chat_turn_stream`

1. 同 once 前置，但調 `chat_turn_stream`。
2. 事件流中新增工具編排事件：
- `ToolCallProposed`
- `ToolExecutionStarted`
- `ToolExecutionCompleted`
- `ToolExecutionFailed`
3. terminal 事件唯一且必達；drop 時必執行 cancel + lease release。

## 6. 三層重構實施序

## Phase L（Local Refactoring：可讀性先行）

1. 更名：
- `types_chat.rs` -> `chat/types.rs`
- `request_normalizer.rs` -> `chat/normalizer.rs`
- `response_normalizer.rs` -> `chat/response.rs`

2. 抽函：
- 自 `gateway.rs` 拆 `prepare_dispatch`, `handle_attempt`, `handle_terminal`。
- 將 adapter 中重覆 `usage` 解析與 stream 終止檢查下沉共用 helper。

3. 去重：
- `openai_compatible` 與 `ollama` 共通 HTTP/stream 錯誤映射抽至 `adapters/openai_compatible/http_common.rs`，並以 shared helper 供 `ollama` 複用。

4. 命名收斂：
- `route`/`route_hint`/`route_ref` 統一語彙（建議 `route_ref`）。

完成判準：

- 不改行為；唯讀性改善；編譯通過。

## Phase S（Structural Refactoring：可維護性）

1. 模組拆分：落實第 3 節拓撲。
2. 邊界重設：
- `gateway.rs` 僅作 capability facade，不再承載 Chat 全流程。
- `chat/service.rs` 為 Chat 能力單一入口。

3. 模式替換：
- 由「單一 `BackendAdapter` trait + dialect map」改為「能力分域 adapter trait」。
- `adapters/mod.rs` 管 provider registry，`chat/backend_contract.rs` 管 Chat adapter trait。

4. 資料流替換：
- 將 `chat_once` 由「讀 stream 聚合」改為「直調 adapter once」。

完成判準：

- `chat_once` 路徑不再觸及 stream 聚合邏輯。
- adapter 與能力模組一一對齊。

## Phase A（Architectural Reset：長期演進）

1. 引入 session/thread store：
- 先落 `InMemoryChatSessionStore`。
- 介面預留持久層替換點（不先實作磁碟版）。

2. 升格 tool orchestration：
- 建 `chat/orchestrator.rs`。
- 將現 `cortex` 內部微循環工具編排下沉至 gateway Chat。

3. 重做 public API：
- 移除舊 `ChatRequest` 主路。
- 新 API 強制 session/thread。

4. 重寫 cortex 調用：
- `core/src/cortex/runtime.rs` 改呼 `AIGateway::chat()` 取得 `ChatGateway`，再經 `ChatSessionHandle/ChatThreadHandle` 發起回合。
- primary 與 helper 以 thread 策略區分：
  - primary：同一 thread 多輪
  - helper：可配置為短線程（ephemeral）

5. 刪舊：
- 刪除 gateway 內 stream->once 聚合碼。
- 刪除舊 trait/object 與過時型別。

完成判準：

- Chat 已真多輪，會話狀態由 gateway 管。
- 工具編排由 gateway 管，不再散於 consumer。
- `chat_once` 由 adapter once 原生提供。

## 7. 檔案變更地圖（預計）

新增：

1. `core/src/ai_gateway/chat/api.rs`
2. `core/src/ai_gateway/chat/service.rs`
3. `core/src/ai_gateway/chat/session_store.rs`
4. `core/src/ai_gateway/chat/orchestrator.rs`
5. `core/src/ai_gateway/chat/policy.rs`
6. `core/src/ai_gateway/chat/backend_contract.rs`
7. `core/src/ai_gateway/adapters/openai_compatible/mod.rs`
8. `core/src/ai_gateway/adapters/openai_compatible/chat.rs`
9. `core/src/ai_gateway/adapters/ollama/mod.rs`
10. `core/src/ai_gateway/adapters/ollama/chat.rs`
11. `core/src/ai_gateway/adapters/github_copilot/mod.rs`
12. `core/src/ai_gateway/adapters/github_copilot/chat.rs`
13. `core/src/ai_gateway/adapters/github_copilot/rpc.rs`

重命名/搬移：

1. `core/src/ai_gateway/types_chat.rs` -> `core/src/ai_gateway/chat/types.rs`
2. `core/src/ai_gateway/request_normalizer.rs` -> `core/src/ai_gateway/chat/normalizer.rs`
3. `core/src/ai_gateway/response_normalizer.rs` -> `core/src/ai_gateway/chat/response.rs`
4. `core/src/ai_gateway/router.rs` -> `core/src/ai_gateway/chat/router.rs`

重寫：

1. `core/src/ai_gateway/gateway.rs`
2. `core/src/ai_gateway/adapters/mod.rs`
3. `core/src/ai_gateway/adapters/openai_compatible.rs`（拆為 provider 目錄，並補 once 原生路徑）
4. `core/src/ai_gateway/adapters/ollama.rs`（拆為 provider 目錄，並補 once 原生路徑）
5. `core/src/ai_gateway/adapters/github_copilot.rs`（拆為 provider 目錄，並補 once 原生路徑）
6. `core/src/cortex/runtime.rs`
7. `core/src/config.rs`
8. `core/beluna.schema.json`

刪除：

1. 舊 `chat_once` 由 stream 聚合之路徑函式
2. 舊通用 adapter 註冊中 Chat 以外無關耦合程式

## 8. 配置改版（破舊）

建議新形：

```json
{
  "ai_gateway": {
    "chat": {
      "default_route": "default",
      "default_max_tool_rounds": 6,
      "default_max_turn_context_messages": 64,
      "default_session_ttl_seconds": 3600,
      "default_turn_timeout_ms": 30000
    },
    "backends": [
      {
        "id": "bailian",
        "chat_adapter": {
          "dialect": "openai_compatible",
          "endpoint": "https://...",
          "credential": {"type": "env", "var": "API_KEY"},
          "models": [{"id": "qwen-plus", "aliases": ["default"]}]
        }
      }
    ]
  }
}
```

命名準則：

1. 凡可被 session/thread/turn 覆寫之預設值，一律 `default_*` 前綴。
2. `default_route` 與其他 `default_*` 同屬「預設策略層」，避免語義不齊。

## 9. Observability 重設（少而精）

目標：以「減日志」增可觀測。INFO 只留決策結果與異常，不作流程逐步廣播；細節下沉 `DEBUG` 與 metrics。

### 9.1 日誌模型（INFO 最小集）

INFO 僅保三類：

1. `chat_turn_summary`（每 turn 恆且僅一條）
2. `chat_turn_anomaly`（僅異常時）
3. `chat_session_lifecycle`（`opened/closed`，僅狀態變化時）

`chat_turn_summary` 必帶鍵：

1. `session_id`
2. `thread_id`
3. `turn_id`
4. `backend_id`
5. `model`
6. `attempts`
7. `latency_ms`
8. `tool_rounds`
9. `usage_in_tokens`
10. `usage_out_tokens`
11. `finish_reason`
12. `outcome`（`ok|failed|cancelled|timeout`）

降級策略：

1. `route_selected`、`tool_round_started/completed`、`attempt_started` 一律降為 `DEBUG`。
2. provider 原始 chunk/stream 細節僅記 `DEBUG`。
3. 同回合內重複語義日志禁止（去重規則：同 `turn_id + event_kind` 只允一次）。

### 9.2 指標模型（以 metrics 補細節）

Task-Local metrics（短生命週期）：

1. `chat_task_latency_ms{task_type=backend_infer|tool_exec, backend, model}`
2. `chat_task_failures_total{task_type, error_kind}`
3. `chat_task_retries_total{backend, model}`

Thread-Local metrics（累積於 thread）：

1. `chat_thread_turns_total{session_id, thread_id}`
2. `chat_thread_tool_calls_total{session_id, thread_id, tool_name}`
3. `chat_thread_tokens_in_total{session_id, thread_id}`
4. `chat_thread_tokens_out_total{session_id, thread_id}`
5. `chat_thread_failures_total{session_id, thread_id, error_kind}`
6. `chat_thread_last_turn_latency_ms{session_id, thread_id}`

落地原則：

1. 高基數維度（`session_id`, `thread_id`）只在本地聚合與日志，預設不上推全域後端。
2. 回合終結時一次性提交 metrics（terminal commit），不做步步 flush。
3. INFO 與 metrics 相互可對帳：任一 `turn_id` 皆可由一條 summary + 一組 terminal 指標還原。

## 10. 驗證矩陣

### 10.1 行為契約

1. session/thread 建立、續用、關閉、TTL。
2. 同 thread 多輪上下文累積與裁剪。
3. tool call 迴圈：成功、失敗、超限、部分失敗補償。
4. `chat_once` 僅走 adapter once 路。
5. `chat_stream` drop 後取消、釋放、終止事件語義。

### 10.2 回歸重點

1. 路由 deterministic（alias 與 `backend/model`）。
2. reliability/budget 既有守衛不退化。
3. telemetry/metrics 欄位不失真（含 session/thread/turn/task 維度）。
4. INFO 級每回合至多一條 summary（異常另加一條），且可獨立還原結果。

## 11. 風險與制策

1. 風險：一次重寫面廣。
- 制策：依 Phase L -> S -> A 三段合併，段段可編譯。

2. 風險：`cortex` 與 Chat orchestration 權責搬移易生倒灌。
- 制策：先定 `ChatToolRuntime` 邊界，再遷移 primary 工具。

3. 風險：配置破壞既有本地環境。
- 制策：提供一份新 `beluna.jsonc` 範本與明確 schema 錯誤訊息。

## 12. 完成判定（Definition of Done）

1. 代碼結構已按能力門/adapter 同形重排。
2. `chat_once` 無任何 stream 聚合模擬路徑。
3. Chat 已具 session/thread/tool orchestration 一級能力。
4. `cortex` 不再自管主要工具編排迴圈。
5. INFO 級日志達「少而精」：每回合一摘要、異常才加筆，且可支援審計。
6. 編譯通過，且 Chat 契約測試覆蓋上述驗證矩陣。
