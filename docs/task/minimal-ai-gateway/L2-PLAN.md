# L2 Plan - Minimal AI Gateway (Index)

- Task Name: `minimal-ai-gateway`
- Stage: `L2` (low-level design)
- Date: 2026-02-08
- Status: `DRAFT_FOR_APPROVAL`

This L2 plan is intentionally split across multiple files.

## File Map

1. `L2-PLAN-01-interfaces-and-modules.md`
- Rust module boundaries, facade API, traits, and execution flow wiring.

2. `L2-PLAN-02-canonical-types-and-event-model.md`
- Canonical request/response structures, tool-call model, event stream invariants, and error taxonomy.

3. `L2-PLAN-03-reliability-budget-credentials-telemetry.md`
- Retry/circuit-breaker semantics, streaming safety rules, budget enforcement algorithms, credential boundaries, telemetry schema.

4. `L2-PLAN-04-backend-adapter-specs.md`
- Per-backend transport+dialect mapping details:
  - OpenAI-compatible (HTTP)
  - Ollama (HTTP)
  - GitHub Copilot SDK/LSP (stdio JSON-RPC)

5. `L2-PLAN-05-config-schema-and-test-plan.md`
- Config struct/schema additions and a concrete test matrix.

## Stage Gate

- This stage is complete only after explicit approval of all L2 files.
- L3 implementation planning will start only after that approval.

Status: `READY_FOR_L2_REVIEW`
