# L3-05 - Test Execution Plan

- Task Name: `minimal-ai-gateway`
- Stage: `L3` detail: verification plan
- Date: 2026-02-08
- Status: `DRAFT_FOR_APPROVAL`

## 1) Test Layers

1. Unit tests
- request normalization invariants,
- deterministic router selection,
- capability guard checks,
- retry/circuit-breaker state transitions,
- budget accounting and concurrency behavior,
- cancellation behavior accounting.

2. Adapter integration tests (mocked)
- OpenAI-compatible SSE fixture,
- Ollama NDJSON fixture,
- Copilot JSON-RPC mock process fixture.

3. End-to-end gateway tests
- event ordering (`Started -> ... -> Completed|Failed`),
- no duplicate terminal events,
- no retry after output/tool events under default policy,
- stream drop triggers underlying cancel and permit release.

## 2) Fixture Strategy

1. HTTP fixtures
- use local mock servers with deterministic payload scripts.

2. Copilot fixture
- spawn fake stdio process that implements minimal JSON-RPC methods used by adapter.

3. Time/retry fixtures
- use controlled tokio time (`pause`/`advance`) for backoff and breaker windows.

## 3) Command Sequence

1. `cargo fmt --check`
2. `cargo test`
3. focused runs for flaky diagnosis if needed:
- `cargo test ai_gateway::tests::reliability`
- `cargo test ai_gateway::tests::openai_compatible`
- `cargo test ai_gateway::tests::copilot_adapter`

## 4) Minimum Acceptance Gate

Implementation is accepted only if:

1. all existing tests remain green,
2. new gateway unit tests are green,
3. adapter integration tests are green,
4. cancellation + retry-safety tests are green,
5. no schema regression in config loading path.

## 5) Known Test Limits (MVP)

1. no live provider-network tests in CI,
2. no throughput/benchmark commitments,
3. no resumable-stream retry path tests.

## 6) Failure Triage Order

1. request normalization and config tests,
2. reliability/cancellation tests,
3. adapter parser tests,
4. e2e stream invariants.

Status: `READY_FOR_L3_REVIEW`
