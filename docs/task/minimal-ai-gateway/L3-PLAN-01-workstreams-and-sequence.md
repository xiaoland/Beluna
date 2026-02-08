# L3-01 - Workstreams and Sequence

- Task Name: `minimal-ai-gateway`
- Stage: `L3` detail: execution sequence
- Date: 2026-02-08
- Status: `DRAFT_FOR_APPROVAL`

## 1) Execution Principles

1. Keep current runtime behavior stable (socket loop unchanged for MVP).
2. Implement gateway as isolated module with compile-time integration only.
3. Enforce strict validation early (`RequestNormalizer`) to avoid provider-specific failure drift.
4. Add tests per workstream before proceeding.

## 2) Ordered Workstreams

### Workstream A - Foundation and Types

Scope:

- add dependencies and module skeleton,
- define canonical types/error taxonomy,
- define traits/interfaces.

Exit criteria:

- module compiles,
- core types and traits unit-tested for serialization/validation basics.

### Workstream B - Config and Strict Validation

Scope:

- extend config structs and schema for `ai_gateway`,
- implement tagged `CredentialRef`,
- add request normalization and message role/linkage checks.

Exit criteria:

- config schema tests pass,
- invalid tool-message states return deterministic `InvalidRequest`.

### Workstream C - Core Gateway Pipeline

Scope:

- router (deterministic, no fallback),
- capability guard,
- budget pre-check and concurrency permits,
- reliability wrapper and canonical stream assembly,
- stream-drop cancellation propagation.

Exit criteria:

- pipeline composes end-to-end with mock adapter,
- cancellation and retry gates verified.

### Workstream D - HTTP Adapters

Scope:

- OpenAI-compatible adapter (`chat/completions`-like),
- Ollama adapter (`/api/chat`),
- stream parsing + graceful degradation.

Exit criteria:

- mock HTTP integration tests pass for both adapters.

### Workstream E - Copilot Adapter (SDK/LSP)

Scope:

- stdio JSON-RPC transport,
- initialize/auth lifecycle,
- completion request mapping,
- cancellation cleanup.

Exit criteria:

- mock process integration tests pass,
- auth-not-ready and protocol-failure mappings verified.

### Workstream F - Final Verification and Docs

Scope:

- full test suite pass,
- docs update,
- write `docs/task/minimal-ai-gateway/RESULT.md`.

Exit criteria:

- all required tests pass,
- result document completed with decisions/deviations/evidence.

## 3) Dependency Graph

1. A -> B -> C
2. C -> D
3. C -> E
4. D + E -> F

## 4) Stop/Go Checkpoints

1. After A: confirm no unresolved type ownership conflicts.
2. After B: confirm strict validation does not break existing config load defaults unexpectedly.
3. After C: confirm retry/cancel semantics before adapter complexity.
4. After D/E: confirm event invariants hold consistently across adapters.

## 5) Boundaries (Out of Scope for This Implementation)

1. multi-backend fallback/routing policies,
2. cost estimation/pricing tables,
3. resumable-streaming retries,
4. live external-provider integration tests in CI.

Status: `READY_FOR_L3_REVIEW`
