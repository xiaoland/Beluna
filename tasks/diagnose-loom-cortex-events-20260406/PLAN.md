# Diagnose Loom Cortex Events After Issue #17

## MVT Core

- Objective & Hypothesis: 找出 issue `#17` 完成後，為什麼 Loom 無法再正確呈現 Cortex 相關事件；初步假設是 `core` 的 AI Gateway / Cortex observability 關聯鍵或事件語義改變，而 `moira` 的投影沒有同步。
- Guardrails Touched: `docs/20-product-tdd/observability-contract.md`; `docs/30-unit-tdd/moira/design.md`
- Verification: 以提交差異、事件欄位流向、Moira 關聯邏輯與必要的本地測試或重現腳本，證明是哪一個欄位/邏輯斷裂造成 Loom 呈現失效。

## Exploration Scaffold

- Perturbation: 使用者回報 issue `#17` 完成後，Loom 無法呈現 Cortex 相關事件。
- Input Type: Reality
- Active Mode or Transition Note: Diagnose
- Governing Anchors: root `AGENTS.md`; `moira/AGENTS.md`; `moira/src/AGENTS.md`; `moira/src-tauri/src/AGENTS.md`; `core/src/cortex/AGENTS.md`; `tasks/moira-v1/*`
- Impact Hypothesis: 若關聯鍵斷裂，Loom chronology 與 Cortex narrative 會遺失或錯掛 AI transport/chat 事件，導致 Cortex investigation 失真。
- Temporary Assumptions: 問題出在 `core -> OTLP -> moira projection` 的契約漂移，而非 DuckDB ingest 全面失效。
- Negotiation Triggers: 若根因跨越 Product TDD 合約變更，或需要放寬/改寫既有 observability 契約，先停下來與使用者確認。
- Promotion Candidates: 若確認是契約或 recurrence tripwire 缺失，補到適當 docs / tests。

## Execution Notes

- key findings:
  - GitHub issue `#17` 本身在 GitHub 上截至 `2026-04-06` 仍是 `Open`，但本地有效落地點是提交 `9224c34` (`2026-04-01`, `ref(core): ai-gateway #17`)。
  - 最新幾個實際 run 在 Moira DuckDB 中都沒有 `cortex.*` 與 `ai-gateway.*` family：例如 `019d62a0-c68e-7ca3-ab3f-4b193f079c9d`（`2026-04-06`）是 `cortex=0`, `ai=0`, `stem.tick=3`；相對地 `019d48ff-3a44-70e3-9498-45d8485e531c`（`2026-04-01`）是 `cortex=31`, `ai=53`。
  - 本機 Moira profile 把 `cortex.primary` route 指到 `bailian` backend，credential 來源是環境變數 `BAILIAN_API_KEY`。
  - 本機 Core log 明確顯示最新 run 在 `primary` 階段失敗：`stage_failed` 之後緊接 `primary_failed_noop`，錯誤是 `missing credential environment variable BAILIAN_API_KEY for backend bailian`。
  - `core/src/cortex/runtime/primary.rs` 的主要 blind spot 是：`ensure_primary_thread()` 先 `open_thread()`；而 `cortex.primary` 的 structured contract event 要到後面的 `run_primary_turn()` 才 emit。當 credential 缺失導致 `open_thread()` 失敗時，系統只留下非 contract 的 warning log，Loom 自然無法重建 Cortex / AI 視圖。
  - issue `#17` 的 thread-lineage observability 變更確實存在，但從目前本機證據看，它不是這次「Loom 看不到 Cortex 事件」的主因；主因是上游根本沒有產生可投影的 `cortex.*` / `ai-gateway.*` 事件。
- decisions made:
  - 將問題定性為「上游 structured event 缺失 + primary bootstrap observability blind spot」，而不是 Lachesis ingest/store 壞掉，也不是目前主導性的 projection mismatch。
  - 先不動 durable docs；目前先保留診斷結果與後續可修補方向。
- final outcome:
  - 直接原因是目前 Moira 使用的 profile 需要 `BAILIAN_API_KEY`，但本機執行時缺少該環境變數，導致 `primary` 在 structured contract event 發出前就失敗。
  - 表面症狀是 Loom 看不到 Cortex 事件；底層原因則是最新 runs 根本沒有 `cortex.*` / `ai-gateway.*` 可供 Lachesis 投影，只剩 `stem.tick` 與若干非 contract warning log。
