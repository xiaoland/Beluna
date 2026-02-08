# L3-02 - File Change Map

- Task Name: `minimal-ai-gateway`
- Stage: `L3` detail: file-level implementation map
- Date: 2026-02-08
- Status: `DRAFT_FOR_APPROVAL`

## 1) Files to Modify

1. `Cargo.toml`
- add crates: `reqwest`, `futures-core`, `futures-util`, `tokio-stream`, `thiserror`, `tracing`, `async-trait`, `uuid`.

2. `src/main.rs`
- add `mod ai_gateway;` for compile linkage.
- do not alter existing runtime loop behavior.

3. `src/config.rs`
- extend `Config` and `RawConfig` with `ai_gateway` section,
- parse and validate new schema fields,
- provide strict typed dialect and credential models.

4. `beluna.schema.json`
- add strict `ai_gateway` schema with tagged `credential` object.

5. `beluna.jsonc`
- add minimal valid `ai_gateway` example for local development.

6. `docs/product/overview.md`
- add concise AI Gateway capability note aligned with implemented scope.

## 2) Files to Add (Gateway Core)

1. `src/ai_gateway/mod.rs`
- module exports and shared type aliases.

2. `src/ai_gateway/types.rs`
- request/response/event/tool/capability structs and enums.

3. `src/ai_gateway/error.rs`
- canonical error kinds + mapping helpers.

4. `src/ai_gateway/gateway.rs`
- `AIGateway` facade and top-level `infer_stream`/`infer_once` orchestration.

5. `src/ai_gateway/router.rs`
- deterministic backend selection and resolved model logic.

6. `src/ai_gateway/request_normalizer.rs`
- strict request normalization and role/linkage validations.

7. `src/ai_gateway/response_normalizer.rs`
- backend event -> canonical event mapping and ordering guards.

8. `src/ai_gateway/capabilities.rs`
- feature checks and unsupported capability error creation.

9. `src/ai_gateway/credentials.rs`
- `CredentialProvider` trait + env provider implementation.

10. `src/ai_gateway/budget.rs`
- budget pre-check, concurrency permits, rate smoothing token bucket, post-usage accounting.

11. `src/ai_gateway/reliability.rs`
- retry/backoff + circuit breaker state handling + cancellation treatment.

12. `src/ai_gateway/telemetry.rs`
- telemetry event types + noop sink.

## 3) Files to Add (Adapters)

1. `src/ai_gateway/adapters/mod.rs`
- adapter registry exports.

2. `src/ai_gateway/adapters/http_common.rs`
- shared HTTP builder, SSE/NDJSON utility parsers, request-id headers.

3. `src/ai_gateway/adapters/openai_compatible.rs`
- `chat/completions`-like mapping and SSE parsing.

4. `src/ai_gateway/adapters/ollama.rs`
- `/api/chat` mapping and NDJSON parsing.

5. `src/ai_gateway/adapters/copilot_rpc.rs`
- stdio JSON-RPC framing, process lifecycle, request/response multiplexing.

6. `src/ai_gateway/adapters/github_copilot.rs`
- Copilot adapter using `copilot_rpc` transport and lifecycle checks.

## 4) Files to Add (Tests)

1. `src/ai_gateway/tests/mod.rs`
- shared test utilities and fixture wiring.

2. `src/ai_gateway/tests/request_normalizer.rs`
- strict message linkage validation tests.

3. `src/ai_gateway/tests/router.rs`
- deterministic selection and no fallback tests.

4. `src/ai_gateway/tests/reliability.rs`
- retry boundaries, circuit breaker, cancellation accounting.

5. `src/ai_gateway/tests/budget.rs`
- concurrency/rate limits and post-usage accounting behavior.

6. `src/ai_gateway/tests/openai_compatible.rs`
- mock SSE parsing and graceful missing-field handling.

7. `src/ai_gateway/tests/ollama.rs`
- mock NDJSON parsing and usage extraction.

8. `src/ai_gateway/tests/copilot_adapter.rs`
- mock JSON-RPC process lifecycle/auth/completion behavior.

9. `src/ai_gateway/tests/gateway_e2e.rs`
- full pipeline with test adapters and event invariants.

## 5) Files to Add (Task Artifact)

1. `docs/task/minimal-ai-gateway/RESULT.md`
- implementation outcome, deviations from L3, test evidence, known limitations.

Status: `READY_FOR_L3_REVIEW`
