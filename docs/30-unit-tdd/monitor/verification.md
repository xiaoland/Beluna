# Monitor Verification

## Behavioral Checks

1. Directory selection loads rows from `core.log.YYYY-MM-DD.N` files.
2. `level` and `target` filters produce deterministic subsets.
3. Timestamp range filtering excludes out-of-window entries.
4. Keyword search matches message and JSON string content.
5. Selecting a row renders expandable JSON tree detail.

## Resilience Checks

1. Malformed NDJSON lines are skipped and counted.
2. Missing optional fields still render row summary without crash.
3. Auto refresh reports observer mode when available and polling fallback otherwise.

## Performance Guardrails (MVP)

1. Memory growth is bounded by max-row cap.
2. UI remains interactive for common local log sizes under capped row window.
3. Full list re-render remains acceptable at capped window size.
