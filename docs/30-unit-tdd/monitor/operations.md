# Monitor Operations

## Startup

1. Load static page and restore persisted filter preferences.
2. Wait for user directory selection.
3. Run initial discovery and parse pass over matching log files.
4. Enable auto refresh mode based on browser capability and user toggle.

## Refresh Loop

1. Discover matching files in selected directory.
2. Incrementally read appended bytes from each file.
3. Parse NDJSON lines with row-level error containment.
4. Recompute filtered view and refresh list/detail panes.

## Shutdown

1. Stop observer or polling timer.
2. Release transient in-memory state on page close/reload.

## Failure Handling

1. Directory picker cancel leaves previous state unchanged.
2. Parsing errors increment counters and do not terminate refresh.
3. Observer setup failure degrades to polling mode.
