# Task Packet: monitor-web-mvp

## Perturbation

Add a new repo/component to improve Beluna observability with a minimal web log viewer. The viewer should list local logs line by line, support field filtering, keyword search, and good JSON browsing. Local log meaning: browser-based local file access with `showDirectoryPicker` and `FileSystemObserver` fallback.

## Input Type

Primary: `Intent`
Secondary: `Constraint` (minimal scope first, browser-only local runtime)

## Governing Anchors

- `AGENTS.md`
- `docs/00-meta/intake-protocol.md`
- `docs/00-meta/promotion-rules.md`
- `docs/20-product-tdd/unit-topology.md`
- `docs/20-product-tdd/cross-unit-contracts.md`
- `core/src/logging.rs`

## Intended Change

Introduce top-level `monitor/` static web app as a read-only consumer of `core` local NDJSON logs, then establish Product TDD + Unit TDD anchors for the new unit.

## Impact Hypothesis

- Primary hit: `monitor/` new web component and associated docs.
- Secondary hit: Product TDD topology and contract docs.
- Confidence: High.
- Unknowns: Browser support variance for `FileSystemObserver` and secure-context requirements.

## Temporary Assumptions

1. Browser runtime is launched on localhost secure context.
2. Log files follow `core.log.YYYY-MM-DD.N` naming pattern.
3. Current core JSON logging shape remains stable for MVP.

## Negotiation Triggers

1. Need backend service for remote aggregation instead of local file reading.
2. Need full-text index or multi-gigabyte pagination as hard requirement.
3. Need compatibility targets that exclude `showDirectoryPicker` without acceptable fallback.

## Acceptance Criteria

1. User can choose `logs/core` directory and see parsed rows.
2. User can filter by `level`, `target`, timestamp range.
3. User can run keyword search over message and full JSON string content.
4. User can inspect one row in expandable JSON tree form.
5. Auto refresh uses `FileSystemObserver` when possible and polling fallback otherwise.
6. Broken JSON lines do not crash UI and are counted.

## Guardrails Touched

- Product TDD topology and cross-unit contract docs.
- Unit TDD docs for monitor.
- No core runtime behavior change.

## Evidence Expected

1. Monitor UI runs at localhost and loads local logs.
2. Filters/search and JSON tree interactions are functional.
3. Docs reflect monitor as a first-class unit/container.

## Outcome

`experiment` (MVP starting point; expect follow-up promotion as behavior stabilizes)

## Promotion Candidates

1. Stable monitor contract around log file consumption semantics.
2. Stable UX defaults (search scope, refresh behavior, malformed-line handling).
