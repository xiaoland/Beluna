# L3-05 - Test Execution Plan
- Task Name: `cortex-mvp`
- Stage: `L3` detail: test plan and execution
- Date: `2026-02-11`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Contract To Test Mapping
### Reactor progression
1. one inbox event -> one cycle result.
2. closed inbox stops reactor gracefully.
3. bounded channel backpressure is enforced.

### Boundedness
4. primary call count is exactly one.
5. subcalls never exceed configured cap.
6. repair attempt count never exceeds one.
7. budget/timeout overflow yields noop.

### World-relative attempts
8. every non-noop attempt includes `attempt_id`.
9. every non-noop attempt includes non-empty `based_on`.
10. feedback ingestion preserves `attempt_id` correlation.

### Clamp + routing
11. unknown affordance is dropped.
12. unsupported capability handle is dropped.
13. payload schema violations are dropped.
14. `max_attempts` stable truncation is deterministic.

### Fallback and recovery
15. first clamp empty + valid repair -> attempts emitted.
16. repair failure -> noop.
17. second clamp empty -> noop.

### Statelessness
18. identical inputs produce identical outputs.
19. no hidden cross-cycle durable goal/commitment state dependency.

### Adapter behavior
20. ai-gateway request limits match reaction limits.
21. tests with mocks perform no network IO.
22. sub-backend capability mismatch results in cycle noop.

## 2) Test File Plan
Add:
1. `/Users/lanzhijiang/Development/Beluna/core/tests/cortex/reactor.rs`
2. `/Users/lanzhijiang/Development/Beluna/core/tests/cortex/clamp.rs`
3. `/Users/lanzhijiang/Development/Beluna/core/tests/cortex/ai_gateway_adapter.rs`

Modify:
4. `/Users/lanzhijiang/Development/Beluna/core/tests/cortex/mod.rs`
5. `/Users/lanzhijiang/Development/Beluna/core/tests/cortex_continuity_flow.rs`
6. `/Users/lanzhijiang/Development/Beluna/core/tests/admission/admission.rs`
7. `/Users/lanzhijiang/Development/Beluna/core/tests/continuity/debits.rs` (if needed by correlation types)

Retire:
8. `/Users/lanzhijiang/Development/Beluna/core/tests/cortex/planner.rs`
9. `/Users/lanzhijiang/Development/Beluna/core/tests/cortex/commitments.rs`

## 3) Execution Commands
```bash
cd /Users/lanzhijiang/Development/Beluna/core
cargo test cortex:: -- --nocapture
cargo test cortex_continuity_flow -- --nocapture
cargo test admission:: -- --nocapture
cargo test continuity:: -- --nocapture
cargo test
```

## 4) Determinism Verification Tactics
1. duplicate-run assertions for same input/output.
2. stable sort assertions by `attempt_id`.
3. deterministic ID golden vectors for attempt and cost attribution ids.
4. no wall-clock/random branching in clamp/reactor tests.

## 5) Acceptance Criteria
1. all newly added reactor/clamp/adapter tests pass.
2. legacy step/planner tests are removed or updated with no stale references.
3. full test suite passes.
4. all L2 contract points have at least one explicit assertion.

Status: `READY_FOR_L3_REVIEW`
