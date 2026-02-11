# RESULT - refactor-mind-cortex-non-cortex

- Date: 2026-02-11
- Status: Completed

## Outcome

`beluna::mind` was removed and replaced by canonical `cortex + non_cortex + spine` modules.

Execution path now follows:

`Cortex -> IntentAttempt[] -> Non-cortex admission -> AdmittedAction[] -> Spine -> ordered events -> Non-cortex reconciliation`

## Implemented Work

1. Module refactor
- Removed `core/src/mind/*` and all `core/tests/mind/*` suites.
- Added `core/src/cortex/*` with:
  - goal/commitment split
  - dynamic scheduling context
  - deterministic `attempt_id` and `cost_attribution_id` derivation
- Added `core/src/non_cortex/*` with:
  - deterministic admission resolver
  - explicit dispositions: `Admitted { degraded } | DeniedHard { code } | DeniedEconomic { code }`
  - deterministic degradation ranking and variant/depth caps
  - survival ledger reservation lifecycle and strict terminality
  - external debit attribution matching and dedupe
  - version tuple in state and ledger entries (`affordance_registry_version`, `cost_policy_version`, `admission_ruleset_version`)
- Added `core/src/spine/*` contracts with:
  - admitted-action-only interface
  - explicit mode enum (`BestEffortReplayable`, `SerializedDeterministic`)
  - settlement-linkable events (`reserve_entry_id`, `cost_attribution_id`)

2. AI Gateway attribution plumbing
- Added `cost_attribution_id` to:
  - `BelunaInferenceRequest`
  - `CanonicalRequest`
  - `GatewayTelemetryEvent` lifecycle variants
- Propagated attribution through gateway execution and telemetry emission.
- Added non-cortex debit source bridge:
  - `AIGatewayApproxDebitSource` (token-based approximate debits).

3. Test migration
- Added new BDT suites:
  - `core/tests/cortex_bdt.rs`
  - `core/tests/non_cortex_bdt.rs`
  - `core/tests/spine_bdt.rs`
  - `core/tests/cortex_non_cortex_flow.rs`
- Added detailed tests for:
  - commitment semantics and failed-status requirements
  - deterministic attempt derivation
  - admission hard/economic outcomes
  - degradation tiebreak/variant cap
  - reservation terminal idempotency and expiry clock
  - attribution-matched external debit behavior
  - admitted-only spine contract and ordered event reconciliation

4. Documentation updates
- Added new feature docs:
  - `docs/features/cortex/*`
  - `docs/features/non-cortex/*`
  - `docs/features/spine/*`
- Added module docs:
  - `docs/modules/cortex/README.md`
  - `docs/modules/non-cortex/README.md`
  - `docs/modules/spine/README.md`
- Added contract docs:
  - `docs/contracts/cortex/README.md`
  - `docs/contracts/non-cortex/README.md`
  - `docs/contracts/spine/README.md`
- Updated indexes and product docs to make `cortex + non-cortex + spine` canonical.
- Updated `core/AGENTS.md` to current module structure.

## Verification

Executed:

```bash
cd /Users/lanzhijiang/Development/Beluna/core
cargo test
```

Result:
- all tests passed.

## Notes

- Spine execution implementation remains contract-level MVP; deterministic noop adapter is used by default/test paths.
- AI Gateway debit feed remains approximate and token-based, intentionally matching MVP scope.
