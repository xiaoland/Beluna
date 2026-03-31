# AI Gateway Refactor Plan (Reference Draft)

> Last Updated: 2026-02-19  
> 说明：本文档仅用于沟通与任务拆解，不是权威规范。最终行为以代码、测试与运行结果为准。

## 1. 背景与目标（参考）

本轮重构先聚焦 3 个方向：

1. 路由以 `backend-id/model-id` 为核心，并支持 alias（如 `default`、`low-cost`）
2. 配置结构以 backend 为首，模型归属 backend 管理
3. 网关对外提供能力化接口（先落 `chat`），替代通用 `infer_once` / `infer_stream` 作为主入口

补充约定（建议）：

- 正常返回可按能力各自建模（不强制统一成功模型）
- 错误保留统一模型（`GatewayError`）

## 2. 重构前状态速记（非权威）

- 目前对外仍以 `infer_once` / `infer_stream` 为主。
- 路由主路径仍是 `default_backend + backend/model override`，alias 路由未形成一等公民。
- 配置里 backend 有 `default_model`，但 model catalog 与 alias map 不完整。

## 3. 目标形态（建议落地）

## 3.1 路由

- 统一目标：`(backend_id, model_id)`
- 调用方可提供：
  - alias：`default` / `low-cost`
  - 直接路由：`<backend-id>/<model-id>`
- 未命中 alias/backend/model 时直接报错，不做隐式 fallback。

## 3.2 配置（backend-first）

建议形态：

- `ai_gateway.backends[]`
  - `id`
  - `dialect`
  - `credential`
  - `endpoint`（以及现有方言配置字段）
  - `models[]`
    - `id`
    - （可选）模型级扩展字段
- `ai_gateway.route_aliases`
  - `alias -> { backend_id, model_id }`

建议约束：

- alias `default` 必须可解析。

## 3.3 API（能力化）

第一阶段先聚焦 Chat：

- `chat_stream(ChatRequest) -> Result<ChatEventStream, GatewayError>`
- `chat_once(ChatRequest) -> Result<ChatResponse, GatewayError>`

说明：

- 允许内部继续复用 canonical pipeline（normalize/route/budget/reliability/adapter），降低改造风险。
- 不保留 `infer_*` 兼容层；统一仅暴露 `chat_*` 能力接口。

## 4. 实施步骤（工作分解）

## Phase A: 文档与范围对齐

- [x] 将本文件重写为“参考草案”语义
- [x] 代码完成后同步 `docs/modules/ai-gateway/*`

## Phase B: 类型与配置

- [x] `types.rs` 增加 backend-owned `models[]`
- [x] 增加 alias 路由类型（例如 `ModelTarget`）
- [x] 增加 chat 能力请求/事件/响应类型
- [x] 更新 `core/beluna.schema.json`

## Phase C: 路由与网关

- [x] `router.rs` 支持 alias + `backend/model` 直达解析
- [x] `gateway.rs` 增加 `chat_stream` / `chat_once`
- [x] 通用 dispatch 下沉为内部实现
- [x] 删除 `infer_*` 与 `BelunaInferenceRequest` 兼容路径

## Phase D: 调用方迁移

- [x] `core/src/cortex/runtime.rs` 从 `infer_once` 迁移到 `chat_once`
- [x] 构造器签名未变化，`core/src/main.rs` 无需改动

## Phase E: 测试与回归

- [x] 更新 `core/tests/ai_gateway/router.rs`（含 alias、unknown alias、unknown model）
- [x] 更新 `core/tests/ai_gateway/gateway_e2e.rs` 到能力化 API
- [x] 运行受影响测试，确认预算/重试/断路器语义不回退

## 5. 完成判定（建议）

满足以下条件可视为本轮完成：

1. 路由支持 alias 与 `backend/model` 双入口
2. 错误路径 deterministic（unknown alias/backend/model 明确报错）
3. 运行时主调用链已使用 `chat_*` 能力接口
4. 现有可靠性与预算行为在测试中保持稳定
5. 相关文档已同步且标注为参考信息

## 6. 风险备注（参考）

- 改动面集中在类型与测试，编译期错误会较多，但可逐步收敛。
- 配置迁移风险主要在 schema 与旧配置兼容处理；建议错误信息尽量指向具体字段。
- 若后续需要 ASR/TTS 等能力，可以复用本次 chat 能力化接口模式扩展。
