# AI Gateway 重构计划

本文档总结了当前 `ai_gateway` 模块的现状、我们就凭证解析（Credential Provider）讨论出的 ADR 要点，以及一个可执行的实现方案与迁移步骤。目标是把凭证来源作为“引用”配置在 `AIGatewayConfig` 中，并在网关内部按 scheme 注册和解析凭证提供者，而不是允许外部直接注入实现。

-- 概要

- 当前实现要点（现状）
  - `AIGateway` 是 core 的一部分，位于 `core/src/ai_gateway/gateway.rs`，提供 `infer_stream` 与 `infer_once` 两种对外调用方式。
  - 凭证相关：存在 `CredentialProvider` trait（`core/src/ai_gateway/credentials.rs`）与内置的 `EnvCredentialProvider`，并且 `AIGateway::new` 当前接受一个 `Arc<dyn CredentialProvider>` 作为外部注入的凭证提供者。
  - 代码其它关键模块：`adapters`（后端适配器 trait）、`router`（后端选择）、`capabilities`（能力检查）、`reliability`（重试/断路器）、`budget`（并发/超时/配额）、`telemetry`。

- 我们讨论的目标（设计意图）
  - 不把 `CredentialProvider` 暴露成外部注入点；相反，在配置里使用对凭证的“引用”语法（例如 `env:MY_VAR`、`cloudflare-secret-store:xiaoland:KEY`），并在 `AIGateway` 内部维护一个按 scheme 注册的 provider registry 来解析这些引用。
  - 目的是把敏感信息从静态 config 中分离，支持多种 secret 后端（env、vault、cloud-provider secret stores 等），并把凭证解析与刷新策略封装在 core 内部实现中。

-- 设计提案（已更新：移除 CredentialProvider）

基于你的决定：移除 `CredentialProvider` 抽象，`credential` 直接存放在 `BackendProfile`（配置）中。本节描述影响、实现建议与迁移要点。

1) 配置表示

- 把 `BackendProfile.credential` 定义为可选字符串（例如 `Option<String>`），示例：
  - `credential: "sk-..."`（内联明文）
  - `credential: "env:OPENAI_API_KEY"`（可保留简单 env 取值语法作为便捷选项）
- 强烈建议文档中标注：在生产环境中不推荐在版本控制或共享配置中使用明文凭证；应限制文件权限并采用部署时注入的配置或加密存储。

1) 移除或简化的组件

- 删除或废弃 `CredentialProvider` trait 及相关实现（例如 `EnvCredentialProvider`），并移除 `AIGateway::new` 中的 `credential_provider` 参数。
- 在 `gateway.rs` 中，直接从 `selected.profile.credential` 构建 `ResolvedCredential`：
  - 若 credential 字符串以 `env:` 前缀，则读取对应环境变量并构建 `auth_header`（但可将此行为视为向后兼容的便利功能）；
  - 若以 `inline:` 或无前缀，则直接把字符串作为 token（或明文）构建 `auth_header`；
  - 否则，若 `credential` 为空或为 `None`，使用 `ResolvedCredential::none()`。

1) 错误与安全策略

- 配置解析阶段需校验 `credential` 格式并在启动时发现明显错误（例如 `env:` 指定的环境变量缺失）。
- 日志中绝不打印凭证内容；对外错误消息应模糊化（例如“missing credential for backend X”）。

1) 测试与迁移

- 更新单元/集成测试以使用内联凭证或 env 前缀的便利方式。
- 在迁移文档中列出替换步骤：将之前依赖 provider 的配置替换为 `credential` 字段的字符串值。

1) 兼容性与回退

- 保留对 `env:` 前缀的支持以减少用户迁移成本，但标注这是“便捷兼容”而非长期推荐的抽象。若后续需要更复杂的 secret 管理，可在另一轮迭代中重新引入 provider/registry（遵循更严格的安全要求）。

-- 具体实现步骤（分阶段）
阶段 1：类型与解析

- 在 `types.rs` 中扩展 `CredentialRef`：新增 `Ref(String)` 或允许 `credential` 字段为 `String`（选择一种）。
- 添加解析工具函数 `parse_credential_ref(raw: &str) -> (scheme, params)`。

阶段 2：Registry 与内置 provider

- 新增 `CredentialProviderRegistry`（模块 `credentials::registry` 或在 `credentials.rs` 内扩展）。
- 在 `AIGateway::new` 中初始化 registry：注册 `env`、`inline`；从 `AIGatewayConfig` 中读取 `credential_providers` 描述并初始化其它 provider（例如 cloudflare），或延后让 ops 在代码中注册（视项目策略）。

阶段 3：将 `AIGateway::new` 的 `credential_provider` 参数移除

- 把外部注入改为内部 registry；修改构造器签名并更新调用方（core 启动代码）以传入 `AIGatewayConfig` 中的 provider 配置段。

阶段 4：解析调用点与错误处理

- 在 `gateway.rs` 中，替换 `credential_provider.resolve(...)` 调用为 registry 的解析函数；保持原有错误包装（`with_backend_id` 等）。
- 添加更明确的错误分类：配置错误 vs 运行时无法获取密钥。

阶段 5：测试、文档与迁移

- 增加单元测试：引用解析、`EnvCredentialProvider` 行为、registry 注册/查找、错误路径。
- 增加集成测试：在 `AIGateway` 初始化和 `infer_stream` 路径上验证凭证解析、缓存与刷新。
- 更新 docs：`docs/task/ai-gateway-refactor/PLAN.md`（本文件）、`docs/overview.md` 的简短说明、以及 `AGENTS.md` 中的相关约定。

-- 配置示例

1) 简单 backend profile（使用 env 引用）

```json
{
  "id": "openai",
  "dialect": "openai_compatible",
  "endpoint": "https://api.openai.com/v1",
  "credential": "env:OPENAI_API_KEY",
  "default_model": "gpt-4"
}
```

1) 声明 provider（可选，全局）

```json
"credential_providers": [
  {
    "scheme": "cloudflare-secret-store",
    "type": "cloudflare",
    "account": "xiaoland",
    "auth": "env:CLOUDFLARE_TOKEN"
  }
]
```

-- 风险与注意事项

- 如果将 provider 设置放在同一配置文件中，必须注意：配置文件本身可能不是安全秘密存储，应避免在配置里放明文 secret（只放 provider 元数据和引用）。
- 提供者初始化（例如 cloud provider 的凭证）仍可能依赖环境或外部身份，需在部署文档中说明启动时的密钥/权限要求。
- 需要兼顾可测试性：保留 `inline` 或测试专用 provider，以便 CI 测试时不依赖真实 secret stores。

## 链式凭证依赖（Chained Credential Providers）

问题描述：在某些 secret store（例如 Cloudflare SecretsStore）场景下，需要一个“主凭证”（master password / master token）来访问次级 secret。这会导致一个 provider 依赖另一个 provider 的解析结果，从而出现链式依赖问题。

可选处理方案：

- 方案 A：禁止链式依赖。强制要求一阶凭证（能直接从 env 或 inline 获得），任何更复杂的 provider 初始化凭证必须写入本地启动环境或配置（明文或加密）。优点：实现简单；缺点：迁移成本高，开发体验差。

- 方案 B（推荐）：支持 provider 初始化时引用其它 provider 的结果，但在初始化阶段进行依赖解析与拓扑排序；禁止循环依赖并对链深做上限。实现细节：
  - provider 配置允许某些字段使用引用语法（如 `auth: env:MASTER_KEY` 或 `auth_ref: provider-scheme:param`）。
  - 在 `AIGateway::new` 初始化 registry 时，先解析 provider 配置中的外部引用，构建依赖图并尝试拓扑排序；若出现循环或无法解析的依赖则报错并拒绝启动（或在可配置的容错模式下降级）。
  - 初始化时把解析到的凭证注入到被依赖的 provider 实例，后续请求使用 provider 本地缓存/刷新策略。优点：启动时能发现配置错误，运行时开销低。

- 方案 C：运行时解析（动态委托）。允许 provider 在运行时调用 registry 去解析依赖（例如在第一次请求时再拉取 master token）。优点：更灵活，可延迟初始化；缺点：更复杂，需要处理运行时错误、重试、并发竞争与安全边界。

- 方案 D：回到极简——允许 inline 明文在 config 中写主凭证（便于单机）。优点：简单易行；缺点：安全风险大，仅适用于开发/本地运行。

推荐：方案 B（初始化期解析与注入）作为默认实现，同时保留方案 D（`inline`）作为 dev/test 便捷选项，并可选支持方案 C 作为高级特性（需明确开启）。

安全与操作注意事项：

- 在初始化解析链式依赖时，绝不在日志中打印密钥或任何敏感字段；只记录解析成功/失败的元信息。
- 明确限制链深（例如 2 或 3），并对循环依赖做检测与失败处理。
- 提供 `allow_provider_chaining` 或类似开关以便在部署策略中显式允许链式解析。
- 文件/配置的访问权限与部署文档要明确说明，推荐生产环境使用专门的 secret manager，而不是把 master key 写在仓库或共享配置里。

迁移与测试要点：

- 单元测试：模拟 provider 链（A -> B -> C），验证拓扑解析、注入行为、循环检测与错误分类。
- 集成测试：在 CI 使用 `inline` provider 模拟 master token 场景，验证 `AIGateway` 启动与 `infer_stream` 流程。
- 添加故障测试：模拟初始化阶段无法解析上游 provider（网络故障、权限不足），确保 `AIGateway` 能给出明确错误并按策略退化或拒绝启动。

实现步骤（补充到阶段 2/3）：

- 在 provider 配置解析器中允许 `auth`/`auth_ref` 字段使用引用语法。
- 在 registry 初始化时先做 provider 配置解析阶段，收集 provider->provider 的依赖边，进行拓扑排序与循环检测。
- 按拓扑顺序初始化 provider，传入已解析的依赖凭证或引用句柄。对无法解析的必需依赖返回配置错误。

此补充旨在兼顾安全性与可操作性：默认采用初始化期解析以尽早发现问题、降低运行时复杂度，同时为开发者保留简单的 `inline` 选择。

-- 后续工作建议

- 如果你同意此方案，我可以：
  1. 在 `core/src/ai_gateway/types.rs` 与 `credentials.rs` 中实现 `Ref(String)` 变体与解析；
  2. 在 `credentials.rs` 中添加 `CredentialProviderRegistry` 并把 `AIGateway::new` 改为在内部构建 registry；
  3. 修改 `gateway.rs` 的调用点以使用 registry；
  4. 添加单元测试与更新文档。

-- 结语
此方案在保持现有功能（向后兼容）的同时，提供了更清晰的秘密管理边界与可扩展的 provider 插件化路径，并且遵循最小暴露原则：把凭证解析责任收敛到 core 内部实现，而不是在运行时把实现注入到网关外部。

----
文件位置：`docs/task/ai-gateway-refactor/PLAN.md`

-- 新的 AI Gateway 配置格式（Backend-first，使用 `connection`）

概要：推荐采用以 `backends` 为中心的配置，能力段（`capabilities`）仅引用 `backend_id` 与 `model_id`，并把与方言/适配器相关的连接参数放入 `connection` 字段，由相应 `apiDialect` 的适配器解析。

建议顶层结构（简洁）：

- `backends`（数组）: 每个 backend 是一个 `BackendProfile`，包含：
  - `id` (必需)
  - `apiDialect` (必需)
  - `credential` (可选)
  - `priority` / `weight` (可选)
  - `capabilities` (可选数组，声明该 backend 支持的能力)
  - `connection` (dialect-specific 连接配置，`serde_json::Value` 或 dialect-specific 结构)
  - `models`（可选全局模型列表，或留到 capability 层）

JSON 示例（摘录）：

```json
{
  "backends": [
    {
      "id": "openai-us",
      "apiDialect": "openai_compatible",
      "credential": "YOUR_OPENAI_API_KEY",
      "priority": 10,
      "capabilities": ["chatLLM", "generateImage"],
      "connection": {
        "base_url": "https://api.openai.com",
        "region": "us"
      }
    }
  ],
}
```

说明：配置格式为 JSON；—— 能力在配置中以 JSON 对象/数组表达，所有连接与方言特定设置均放入 `backends[].connection`，能力段只包含对 `backend_id` 与 `model` 的引用和覆盖字段。

校验与注意事项（要点）：

- 启动验证：确保每个 `capabilities` 条目引用的 `backend_id` 存在且 `backends[].capabilities` 包含所需能力；适配器负责解析并校验 `connection` 字段的必需子字段。
- 安全：强制文件不入 VCS、在文档中标注不推荐内联凭证；支持 `env:` 与 `file:` 前缀作为便捷安全选项。
- 由 `backend` 聚合配置有利于健康检查、计费、路由策略与统一运维。能力层仅保留最小引用信息，避免重复与配置不一致。

迁移建议（概览）：

- 把现有 capability-level 后端字段抽取到 `backends[]`；在 capability 段替换为 `backend_id` 引用。
- 在启动阶段加入一致性校验（存在性、capability 支持、`connection` 基本字段）。

-- 能力化接口：ADR 与架构层面变更（摘要）

目标：将 AI Gateway 的表面接口由通用的 `infer_once`/`infer_stream` 能力化为面向具体 AI 能力的清晰 API（例如 `chat_llm_once`、`chat_llm_stream`、`asr_stream`），并通过 `backend-id/model-id` 指定能力提供方与模型。

关键变更要点：

- API 层：在 `AIGateway` 上添加能力化入口方法（示例签名）：
  - `chat_llm_stream(&self, backend_id: Option<BackendId>, model: Option<String>, req: ChatRequest) -> Result<GatewayEventStream, GatewayError>`
  - `chat_llm_once(&self, backend_id: Option<BackendId>, model: Option<String>, req: ChatRequest) -> Result<CanonicalFinalResponse, GatewayError>`
  - `asr_stream(&self, backend_id: Option<BackendId>, model: Option<String>, req: ASRRequest) -> Result<GatewayEventStream, GatewayError>`
  这些方法内部可构造或映射到现有的 `BelunaInferenceRequest`/`CanonicalRequest` 并复用现有 dispatch/adapter 流程以避免大量重复实现。

- 类型层：在 `types.rs` 增补“能力”相关类型的最小描述（`Capability` 枚举或按位字段）和能力专属的请求/响应模型（`ChatRequest/ChatResponse`、`ASRRequest/ASREvent` 等），以提供更强的类型文档与编译时检查。

- 能力声明与校验：扩展 `BackendCapabilities`，并在 `CapabilityGuard::assert_supported` 中增加能力检查（后端是否支持 Chat/ASR/TTS 等）。

- 适配器与后端契约：适配器继续实现 `BackendAdapter::invoke_stream`，但 `static_capabilities()` 必须包含能力集合；适配器内部映射能力化请求到后端协议。

- 路由与选择：保留 `backend-id/model-id` 语义；能力化入口将这些参数映射为 `backend_hint`/`model_override`，由现有 `BackendRouter` 选择后端并解析 model。

兼容性与迁移策略（高层）：

- 保留现有 `infer_once`/`infer_stream` 作为通用/向后兼容入口，标注为“通用兼容接口”。
- 首先在 `types.rs` 添加能力类型与 `BackendCapabilities` 扩展；其次在 `AIGateway` 增加能力化方法（实现可先委托至 `infer_*`）；最后逐步在适配器中暴露能力支持的示例与文档。

设计权衡与注意事项：

- 优点：能力化接口使上层调用者语义清晰、便于权限/配额/遥测按能力细分；便于未来接入能力级路由与策略。
- 成本：新增类型与接口会扩充 public API，需保证向后兼容并分阶段推广。
- 实施要点：优先采取“轻量能力化”——最小类型补充与 API 适配器层映射，避免一开始就拆分大量适配器实现。

在 `PLAN.md` 的实现任务中我已把该 ADR 列为架构级变更，下一步可把这部分变更拆成 2-3 个 PR：类型定义、AIGateway API、适配器能力注释与测试。
