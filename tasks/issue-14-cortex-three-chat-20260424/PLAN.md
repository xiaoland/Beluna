# Issue #14 Implementation Draft

> Last Updated: 2026-04-25
> Status: working draft
> Scope: non-authoritative task packet for implementation alignment before coding
> Related issues: `#14`, `#27`

## MVT Core

- Objective & Hypothesis: 將 `Primary / Attention / Cleanup` 的高層決策收斂成可直接進入實作的 runtime 草案，先清楚定義模組邊界、phase contract、tool 面與 state application，再進入 `#27` 的骨架 cleanup 與 `#14` 的功能重構。
- Guardrails Touched:
  - `core` 是 runtime authority owner；cross-chat ownership 必須在 runtime code 中真實成立，不能只靠 prompt 或命名成立。
  - `reset-context` 只重置 Primary thread history；`goal_forest` 是 `CognitionState` 的一部分，不得把它誤當成 thread-local context。
- Verification:
  - 本 packet 對以下事項提供明確草案：模組拓樸、phase API、tool schema、orchestrator 流程、Attention/ Cleanup application semantics、`AfferentRuleControlPort` 變更方向。
  - 草案內容與 `#14` comment / description 已確認的設計決策一致，且不再保留未定義的關鍵空洞。
  - 初期實作不新增 tests；驗證先以編譯、局部 smoke 檢查與人工 review 為主，等 phase shape 穩定後再補測試。

## Exploration Scaffold

- Input Type: `Artifact`
- Active Mode or Transition Note: `Explore -> Execute-prep`; 目前是在 coding 前收斂 implementation shape，不進入 code changes。
- Governing Anchors:
  - root [AGENTS.md](/Users/lanzhijiang/Development/Beluna/AGENTS.md)
  - [core/AGENTS.md](/Users/lanzhijiang/Development/Beluna/core/AGENTS.md)
  - [core/src/cortex/AGENTS.md](/Users/lanzhijiang/Development/Beluna/core/src/cortex/AGENTS.md)
  - [docs/30-unit-tdd/core/design.md](/Users/lanzhijiang/Development/Beluna/docs/30-unit-tdd/core/design.md)
  - [docs/30-unit-tdd/core/interfaces.md](/Users/lanzhijiang/Development/Beluna/docs/30-unit-tdd/core/interfaces.md)
  - GitHub issues `#14` and `#27`
- Temporary Assumptions:
  - 第一刀先不拆獨立 `derive.rs`；derive helper 先留在 orchestrator 內部，避免為對稱而對稱。
  - 第一階段 attention / cleanup 各自 fresh derive，Primary 保持唯一長壽 thread。
  - `patch-goal-forest` 的 tool payload contract 是 `operations[]`，但 phase output 攜帶的是 canonical next-state `GoalNode[]`。
- Negotiation Triggers:
  - 若後續實作發現 `break-primary-phase` 必須帶參數才能消除歧義，需先回到 issue 討論。
  - 若 attention full-state ownership 無法以 `replace_ruleset` 類接口落地，需先回到 issue 討論，不回退成 increment-only owner。
- Promotion Candidates:
  - 若本 packet 中的 phase contract、tool contract、port contract 穩定並已經落地，之後應分別提升到 `docs/20-product-tdd` / `docs/30-unit-tdd/core`。

## Current-State Read

- 現況的單體實作主要集中在 [core/src/cortex/runtime/primary.rs](/Users/lanzhijiang/Development/Beluna/core/src/cortex/runtime/primary.rs)。
- 目前 `PrimaryToolExecutor` 同時持有：
  - dynamic act dispatch
  - sleep
  - afferent rule add/remove
  - goal-forest patch
  - `reset_context`
  - Primary thread rewrite
- 這表示 runtime orchestration、Primary cognition、attention-like control、cleanup-like maintenance 仍然沒有被拆成真實 owner。

## Confirmed Decisions Snapshot

- Primary 必須透過無參數 `break-primary-phase` 顯式結束本 tick 的 Primary phase。
- 若同一 turn 內既有 act tool call，又有 `break-primary-phase`：
  - 先派發該 turn 內 act tool call
  - 再套用 break
- `wait-for-sense` 從 Primary 移除，改由 Attention / afferent gating 路徑承接。
- Attention 也走工具面，不走自由文字解析。
- Attention 若未輸出新的 gating state，runtime 必須保留當前已生效的 afferent ruleset。
- Cleanup 工具面為：
  - `patch-goal-forest`
  - `reset-context`
- 目前不做 `rewrite-context`。
- `reset-context` 的語義是：清空 Primary 歷史，只保留 system prompt。
- `patch-goal-forest` 的 payload contract 是 `operations[]`。
- Attention / Cleanup 具有獨立 route / model 配置。
- Attention / Cleanup 每個 admitted tick 都 fresh derive 一個新 thread。
- `patch-goal-forest.operations[]` 直接由 deterministic Rust reducer 歸約，不再呼叫 LLM helper 解讀 patch instruction。
- singleton control tools 的重複呼叫語義採 fail-closed。
- route config 刪除 `helper_routes` 概念，目標 route 欄位只保留 `primary / sense_helper / acts_helper / attention / cleanup`。
- 初期實作不新增 tests。

## Route Config Target

現況的 `CortexHelperRoutesConfig` 以 `helper_routes` 命名，且包含：

- `default`
- `primary`
- `sense_helper`
- `acts_helper`
- `goal_forest_helper`

目標設計刪除 `helper_routes` 作為概念，因為 `Primary / Attention / Cleanup` 都不是 helper。建議保留一層 route grouping，例如：

```rust
pub struct CortexRoutesConfig {
    pub primary: Option<String>,
    pub sense_helper: Option<String>,
    pub acts_helper: Option<String>,
    pub attention: Option<String>,
    pub cleanup: Option<String>,
}
```

設計語義：

- `goal_forest_helper` 移除，因為 `patch-goal-forest.operations[]` 由 deterministic Rust reducer 處理。
- `default` 移除，避免不同 cognition owner 意外共享 route 決策。
- 欄位不建議直接攤平到 `CortexRuntimeConfig` 頂層；保留 `routes` 或 `organ_routes` 分組能讓 runtime capacity / limits / routes 的責任更清楚。

## Target Module Topology

第一刀預期拓樸：

```text
core/src/cortex/runtime/
├── mod.rs
├── orchestrator.rs
├── apply.rs
├── primary/
│   ├── mod.rs
│   ├── session.rs
│   ├── runner.rs
│   └── tools.rs
├── attention/
│   ├── mod.rs
│   ├── runner.rs
│   ├── tools.rs
│   └── schema.rs
└── cleanup/
    ├── mod.rs
    ├── runner.rs
    └── tools.rs
```

### Ownership by Module

- `orchestrator.rs`
  - 一個 admitted tick 的 phase sequencing
  - Primary 執行
  - fresh derive Attention / Cleanup
  - 並行等待結果
  - 呼叫 apply layer

- `apply.rs`
  - 將 Attention / Cleanup phase output 轉成 runtime-owned side effects
  - 保持「chat 決策」與「runtime mutation」分離

- `primary/session.rs`
  - Primary 的長壽 thread
  - Primary continuation state
  - Cleanup reset 後的 thread 生命週期管理

- `primary/runner.rs`
  - 跑 Primary phase，直到顯式 `break-primary-phase`
  - 只處理 Primary 該有的 cognition + tool turn loop

- `primary/tools.rs`
  - `expand-senses`
  - dynamic act tools
  - `break-primary-phase`

- `attention/runner.rs`
  - 從 committed Primary history fresh derive Attention thread
  - 執行 Attention phase
  - 透過 Attention tools 收集 gating / sleep 決策

- `attention/tools.rs`
  - Attention 工具面

- `cleanup/runner.rs`
  - 從 committed Primary history fresh derive Cleanup thread
  - 執行 Cleanup phase
  - 透過 Cleanup tools 收集 maintenance actions

- `cleanup/tools.rs`
  - `patch-goal-forest`
  - `reset-context`

## Runtime Phase Contracts

```rust
pub struct PrimaryPhaseOutput {
    pub committed_thread: Thread,
    pub committed_turn_id: u64,
}

pub struct AttentionGatingState {
    pub deferral_rules: Vec<DeferralRuleSpec>,
}

pub struct AttentionPhaseOutput {
    pub gating_state: Option<AttentionGatingState>,
    pub sleep_ticks: Option<u64>,
}

pub struct CleanupPhaseOutput {
    pub patched_goal_forest: Option<Vec<GoalNode>>,
    pub reset_context_requested: bool,
}
```

### Contract Notes

- `PrimaryPhaseOutput`
  - 不帶 `dispatched_act_count`
  - 不帶 acts list
  - Attention / Cleanup 如需 act trace，直接從 committed Primary history 讀 tool result

- `AttentionPhaseOutput.gating_state`
  - `Some(_)` 表示本 tick 提供新的完整 gating state，runtime 應 replace
  - `None` 表示本 tick 不提供新的 gating state，runtime 保持現有 ruleset

- `CleanupPhaseOutput.patched_goal_forest`
  - `Some(_)` 表示本 tick 對 goal-forest 有維護結果
  - `None` 表示本 tick 對 goal-forest 無變更

## Tool Contracts

## Primary Tools

### `break-primary-phase`

```rust
name: "break-primary-phase"
input_schema: {}
```

語義：

- 顯式表達「本 tick 的 Primary phase 已結束」
- 本 turn 內若同時有 act tool call，先派發 act，再套用 break
- 若 Primary turn commit 時沒有 pending continuation 且也沒有 `break-primary-phase`，視為 protocol violation

### Dynamic Act Tools

act tools 不再包含 `wait_for_sense` 參數。最小返回面應保留：

- `act_instance_id`
- `might_emit_sense_ids`
- `dispatch_result`

這些資訊供 Attention 之後從 committed history 中讀取。

## Attention Tools

第一版預期最小工具面：

### `replace-afferent-gating`

```rust
{
  "rules": [DeferralRuleSpec, ...]
}
```

語義：

- 用新的完整 ruleset replace attention-owned current ruleset
- 若本 tick 沒有呼叫此工具，runtime 保持現狀

### `sleep`

```rust
{
  "ticks": u64
}
```

語義：

- 生成 runtime sleep signal
- 不再由 Primary 持有

## Cleanup Tools

### `patch-goal-forest`

```rust
{
  "operations": [GoalForestPatchOperation, ...]
}
```

語義：

- tool payload contract 是 `operations[]`
- Cleanup tool executor 在 phase 內用 deterministic Rust reducer 將 operations 歸約為 canonical 的 `Vec<GoalNode>`
- `CleanupPhaseOutput` 攜帶的是 `patched_goal_forest: Option<Vec<GoalNode>>`
- 重複呼叫採 fail-closed

### `reset-context`

```rust
{}
```

語義：

- 僅設置 `reset_context_requested = true`
- 真正的 thread reset 在 `apply_cleanup_result(...)` 中發生

## Goal-Forest Patch Operation Sketch

此處只列 shape 草圖，不是最終 schema 文案：

```rust
enum GoalForestPatchOperation {
    AddRoot { node: GoalNode },
    ReplaceNode { node_id: String, node: GoalNode },
    RemoveNode { node_id: String },
    InsertChild { parent_id: String, index: Option<usize>, node: GoalNode },
    ReplaceChildren { parent_id: String, children: Vec<GoalNode> },
    UpdateFields {
        node_id: String,
        status: Option<String>,
        weight: Option<f64>,
        summary: Option<String>,
    },
}
```

限制原則：

- operation target 必須靠 `node_id` 對齊，不引入額外 numbering owner
- phase 內歸約後必須產生完整合法 `GoalNode[]`
- invalid operation sequence 應 fail closed，而不是隱式修復
- reducer 必須 deterministic；不呼叫 LLM helper 或 prompt-driven patcher

## Orchestrator Flow

```rust
pub async fn run_tick(&self, tick: TickContext) -> Result<CortexOutput, CortexError> {
    let primary = self.primary.run_phase(&tick).await?;

    let (attention, cleanup) = tokio::join!(
        self.attention.run_phase(&tick, &primary),
        self.cleanup.run_phase(&tick, &primary),
    );

    self.apply_attention_result(attention?).await?;
    self.apply_cleanup_result(cleanup?).await?;

    Ok(CortexOutput {
        control: CortexControlDirective::default(),
        pending_primary_continuation: false,
    })
}
```

### Phase Semantics

- Primary 是本 tick 的第一階段
- 只有當 Primary 已 commit 且已顯式 break，Attention / Cleanup 才能啟動
- Attention / Cleanup 基於相同的 Primary committed history fresh derive
- 兩者並行，不互相依賴
- runtime 最後統一 apply phase outputs

## Apply Layer

## `apply_attention_result(...)`

語義：

1. 若 `gating_state = Some(_)`
   - replace 當前 afferent ruleset
2. 若 `gating_state = None`
   - 保持當前 ruleset，不覆蓋
3. 若 `sleep_ticks = Some(n)`
   - 更新 runtime sleep gate

## `apply_cleanup_result(...)`

語義：

1. 若 `patched_goal_forest = Some(nodes)`
   - persist 到 `CognitionState.goal_forest`
2. 若 `reset_context_requested = true`
   - 清空 Primary 歷史
   - 只保留 system prompt

注意：

- `reset-context` 作用於 Primary thread history，不作用於 cognition state
- `goal_forest` 會在下一個 tick 因為重新投影 input IR 而再次進入 Primary
- 先 patch 再 reset，主要是讓本 tick 的 maintenance 結果先落成 durable cognition state，再做 thread-level cleanup

## Primary Session

```rust
pub struct PrimarySession {
    pub thread: Thread,
    pub continuation: Option<PrimaryContinuationState>,
}
```

語義：

- Primary 是唯一長壽 thread
- Attention / Cleanup 都不是長壽 thread
- `reset-context` 發生時：
  - continuation 清空
  - Primary thread history 清空
  - system prompt 保留

## Port Changes

`AfferentRuleControlPort` 需要新增完整 state owner 友善的接口。

最小方向：

```rust
async fn replace_ruleset(&self, rules: Vec<DeferralRuleSpec>) -> Result<u64, Error>;
```

保留 `add_rule/remove_rule` 與否可在實作時再決定；但 Attention 自己不應以 increment-only API 當 owner。

## Singleton Control Tool Policy

以下工具每個 phase/tick 最多接受一次有效呼叫；重複呼叫直接 protocol error：

- `break-primary-phase`
- `replace-afferent-gating`
- `sleep`
- `reset-context`
- `patch-goal-forest`

## Suggested Execution Order

建議落地順序：

1. 在 `#27` 中先抽出 `PrimarySession`
2. 刪除 `helper_routes` 概念，建立新的 route config shape
3. 引入 `break-primary-phase`，讓 Primary phase 有顯式合法結束條件
4. 將 Cleanup tools 從現有 Primary tool 路徑拆出，建立 phase-local accumulator
5. 將 `patch-goal-forest.operations[]` reducer 做成 deterministic Rust reducer
6. 將 Attention 從 Primary tool 路徑拆出，建立獨立 Attention tools 與 phase output
7. 補 `apply.rs`，把 runtime side effects 從 tool executor 中抽離

## Verification Draft for Later Implementation

- Primary 沒有 `break-primary-phase` 就 commit，應 fail closed
- 同一 turn 有 act tool call + break 時，act 先派發，break 後套用
- Attention 無 `replace-afferent-gating` 時，不覆蓋現有 ruleset
- Attention 的 `sleep` 會更新 runtime sleep gate
- Cleanup `patch-goal-forest` 會歸約成 canonical `GoalNode[]`
- Cleanup `reset-context` 後，Primary history 清空，但 system prompt 保留
- Cleanup `reset-context` 不會修改 cognition state 中已持久化的 goal forest
- Attention / Cleanup 每 tick 都是 fresh derive，不保留長壽 thread state

目前不新增 tests。上述驗證點保留作為後續補測試的候選清單；初期實作先以 `cargo check --lib`、局部手動檢查與 code review 驗證。

## Execution Notes

- key findings:
  - `#14` 的 blocker 不是單一工具或 prompt，而是 phase ownership 尚未在 runtime code 中成立。
  - `goal_forest` 與 `context` 必須嚴格區分：前者屬於 cognition state，後者屬於 Primary thread history。
  - Attention 如要成為 full-state owner，port contract 必須支持 replace semantics。
- decisions made:
  - `break-primary-phase` 採無參數工具。
  - Attention 也走工具面。
  - `patch-goal-forest` payload contract 採 `operations[]`，並由 deterministic Rust reducer 歸約。
  - `reset-context` 只清空 Primary 歷史，只保留 system prompt。
  - singleton control tools 重複呼叫採 fail-closed。
  - route config 刪除 `helper_routes` 概念，目標欄位為 `primary / sense_helper / acts_helper / attention / cleanup`。
  - 初期不新增 tests。
- final outcome:
  - 本 packet 提供了一版可直接進入 `#27` cleanup 與 `#14` coding 前確認的實作草案。
