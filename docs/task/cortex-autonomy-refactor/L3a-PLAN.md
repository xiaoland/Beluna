# L3a Plan - Execution Checklist
- Task: `cortex-autonomy-refactor`
- Stage: `L3a`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Implementation Checklist
1. Add cortex cognition domain types and export wiring.
2. Introduce unified prompt module and migrate all prompt literals.
3. Refactor IR tags and parsers for goal-tree/l1-memory contracts.
4. Add goal-tree input helper and cache (user partition only).
5. Refactor output helpers to direct-array schemas and parsers.
6. Apply goal-tree/l1-memory patch ops inside cortex to produce full new cognition state.
7. Implement continuity persistence module with atomic JSON read/write.
8. Replace continuity state logic with full-state validation+persist guardrails.
9. Remove continuity spine-event recording paths.
10. Implement continuity middleware `on_act` as current no-op continue.
11. Refactor stem into modeful scheduler with 1s tick + skip missed behavior.
12. Add sleep act interception with `seconds` payload and sleeping mode transition.
13. Replace dispatch flow with per-act middleware chain (`continuity.on_act -> spine.on_act`).
14. Remove ledger dispatch path from stem.
15. Add spine middleware entry `on_act` and failure->sense emission path.
16. Update main shutdown to enqueue `Sense::Hibernate`.
17. Extend config and JSON schema (`tick_interval_ms`, missed behavior, continuity state path).
18. Update tests for new sense/control/IR/dispatch contracts.
19. Run `cargo build` in `core/` and fix compilation issues.
20. Update docs (features/modules/contracts/agents/overview/glossary).
21. Write `RESULT.md` with executed commands and outcomes.

## 2) Non-Negotiable Guardrails During Implementation
1. No abstraction layer for continuity persistence.
2. No reintroduction of `Sense::Sleep` control sense.
3. No wrapper duplication in output helper JSON schemas.
4. No confusion between output IR and final cortex boundary output.
5. No root partition mutation via runtime patches.

## 3) Completion Gate
All checklist items complete and `cargo build` passes.

