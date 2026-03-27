# Monitor Interfaces

## External Interface

1. Browser UI entrypoint:

- Open `monitor/index.html` via localhost static server.

1. Directory access interface:

- `showDirectoryPicker` for local directory selection.
- `FileSystemFileHandle`/`File` APIs for incremental file reads.

1. Refresh interface:

- `FileSystemObserver` when available.
- Polling fallback when observer is unavailable.

## Consumed Contract

1. Core local log files in `logging.dir` with NDJSON rows.
2. Log row JSON fields include core-produced metadata (`timestamp`, `level`, `target`, `fields`, optional `span`/`spans`).
3. File name pattern consumed by default: `core.log.YYYY-MM-DD.N`.
