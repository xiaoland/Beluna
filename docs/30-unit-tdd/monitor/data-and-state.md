# Monitor Data And State

## Owned State

1. Selected directory handle and refresh mode.
2. In-memory parsed log entry window.
3. Per-file incremental cursor (`offset`, `line`, `remainder`).
4. Current filter/search criteria and selected row identity.
5. Parse error counters for malformed rows.

## Consumed State

1. Local NDJSON log file content produced by core.
2. Browser file metadata and file text chunks.
3. Browser local storage for persisted filter preferences.

## Local Invariants

1. Entry ID uniqueness is derived from `(file, line)`.
2. Filtering/search is pure over current in-memory window.
3. Max-row setting bounds memory growth and render cost.
4. Malformed rows are skipped and counted, not rendered as valid entries.
